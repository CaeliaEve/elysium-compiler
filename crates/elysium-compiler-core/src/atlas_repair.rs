use crate::json_ext::{optional_value_string, optional_value_u64, value_string, value_u64};
use serde_json::{json, Value};
use std::collections::BTreeMap;

fn item_id_from_asset_id(asset_id: &str) -> Option<String> {
    asset_id.strip_prefix("nesqlpp:item/").map(str::to_string)
}

fn is_buildcraft_facade(item: &Value) -> bool {
    let family = value_string(item, "semanticFamily")
        .or_else(|| value_string(item, "family"))
        .unwrap_or_default()
        .to_ascii_lowercase();
    let item_mod_id = value_string(item, "modId")
        .unwrap_or_default()
        .to_ascii_lowercase()
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
        .collect::<String>();
    let item_internal_name = value_string(item, "internalName")
        .unwrap_or_default()
        .to_ascii_lowercase();
    family == "facade.buildcraft"
        || (item_mod_id == "buildcrafttransport" && item_internal_name == "pipefacade")
}

fn exact_facade_source<'a>(
    target: &Value,
    atlas_by_asset: &'a BTreeMap<String, Value>,
    atlas_by_item: &'a BTreeMap<String, Value>,
) -> Option<&'a Value> {
    let source_asset_id = value_string(target, "sourceAssetId").filter(|value| !value.is_empty());
    let source_item_id = value_string(target, "sourceItemId").filter(|value| !value.is_empty());
    let by_asset = source_asset_id
        .as_ref()
        .and_then(|asset_id| atlas_by_asset.get(asset_id));
    let by_item = source_item_id
        .as_ref()
        .and_then(|item_id| atlas_by_item.get(item_id));

    match (source_asset_id, source_item_id, by_asset, by_item) {
        (Some(_), Some(_), Some(asset_entry), Some(item_entry))
            if value_string(asset_entry, "itemId") == value_string(item_entry, "itemId")
                && value_string(asset_entry, "assetId") == value_string(item_entry, "assetId") =>
        {
            Some(asset_entry)
        }
        (Some(_), None, Some(asset_entry), _) => Some(asset_entry),
        (None, Some(_), _, Some(item_entry)) => Some(item_entry),
        _ => None,
    }
}

fn valid_resolved_facade_target(target: &Value) -> bool {
    value_string(target, "status").as_deref() == Some("resolved")
        && value_string(target, "reason").is_some_and(|value| !value.trim().is_empty())
        && value_string(target, "blockRegistryName").is_some_and(|value| !value.trim().is_empty())
        && target
            .get("meta")
            .and_then(Value::as_i64)
            .is_some_and(|value| value >= 0)
        && value_string(target, "sourceItemId").is_some_and(|value| !value.trim().is_empty())
        && value_string(target, "sourceAssetId").is_some_and(|value| !value.trim().is_empty())
}

fn expected_facade_status(target_count: u64, resolved_target_count: u64) -> Option<&'static str> {
    if resolved_target_count > target_count {
        None
    } else if target_count > 0 && resolved_target_count == target_count {
        Some("resolved")
    } else if resolved_target_count > 0 {
        Some("partial")
    } else {
        Some("unresolved")
    }
}

fn validated_facade_targets<'a>(
    resolution: &'a Value,
    status: &str,
) -> Result<&'a [Value], String> {
    let targets = resolution
        .get("targets")
        .and_then(Value::as_array)
        .ok_or_else(|| "targets must be an array".to_string())?;
    let target_count = value_u64(resolution, "targetCount")
        .ok_or_else(|| "targetCount must be an unsigned integer".to_string())?;
    let resolved_target_count = value_u64(resolution, "resolvedTargetCount")
        .ok_or_else(|| "resolvedTargetCount must be an unsigned integer".to_string())?;
    if target_count != targets.len() as u64 {
        return Err("targetCount must equal targets.length".to_string());
    }
    for (index, target) in targets.iter().enumerate() {
        if value_u64(target, "targetIndex") != Some(index as u64) {
            return Err("targetIndex values must be contiguous and ordered from zero".to_string());
        }
        if !matches!(
            value_string(target, "status").as_deref(),
            Some("resolved" | "unresolved")
        ) {
            return Err("target status must be resolved or unresolved".to_string());
        }
        if !value_string(target, "reason").is_some_and(|value| !value.trim().is_empty()) {
            return Err("every target must have a non-empty reason".to_string());
        }
    }
    let actual_resolved_target_count = targets
        .iter()
        .filter(|target| valid_resolved_facade_target(target))
        .count() as u64;
    if resolved_target_count != actual_resolved_target_count {
        return Err(
            "resolvedTargetCount must equal the number of structurally valid resolved targets"
                .to_string(),
        );
    }
    if expected_facade_status(target_count, resolved_target_count) != Some(status) {
        return Err(
            "status must be uniquely derived from targetCount and resolvedTargetCount".to_string(),
        );
    }
    Ok(targets)
}

fn facade_issue(
    code: &str,
    item_id: &str,
    asset_id: &str,
    status: Option<&str>,
    reason: impl Into<String>,
) -> Value {
    json!({
        "code": code,
        "itemId": item_id,
        "assetId": asset_id,
        "facadeResolutionStatus": status,
        "reason": reason.into(),
        "recommendedFix": "NESQL++ must export one authoritative facadeResolutions row with an exact facade item/asset identity and at least one exact source item/asset target; compiler-side NBT or localized-name inference is forbidden.",
    })
}

pub struct AtlasRepairResult {
    pub items: Vec<Value>,
    pub repaired_facades: u64,
    pub unresolved_facades: Vec<Value>,
}

pub fn repaired_browser_atlas_items(
    atlas: &Value,
    texture_rows: &[Value],
    item_rows: &[Value],
    facade_resolution_rows: &[Value],
) -> AtlasRepairResult {
    let mut items = atlas
        .get("items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut existing = items
        .iter()
        .filter_map(|entry| value_string(entry, "itemId").map(|item_id| (item_id, true)))
        .collect::<BTreeMap<_, bool>>();

    for texture in texture_rows {
        let Some(asset_id) = value_string(texture, "assetId") else {
            continue;
        };
        let Some(item_id) = item_id_from_asset_id(&asset_id) else {
            continue;
        };
        if existing.contains_key(&item_id) {
            continue;
        }
        let Some(atlas_file) = value_string(texture, "atlasFile") else {
            continue;
        };
        let rect = texture.get("rect").cloned().unwrap_or_else(|| {
            json!({
                "x": 0,
                "y": 0,
                "width": 16,
                "height": 16,
            })
        });
        let x = rect.get("x").and_then(Value::as_u64).unwrap_or(0);
        let y = rect.get("y").and_then(Value::as_u64).unwrap_or(0);
        let width = rect.get("width").and_then(Value::as_u64).unwrap_or(16);
        let height = rect.get("height").and_then(Value::as_u64).unwrap_or(16);
        items.push(json!({
            "itemId": item_id,
            "assetId": asset_id,
            "hasStaticAtlas": true,
            "resolutionMode": "rust_texture_row_repair",
            "staticAtlas": {
                "atlasFile": atlas_file,
                "atlasWidth": x + width,
                "atlasHeight": y + height,
                "x": x,
                "y": y,
                "width": width,
                "height": height,
            },
        }));
        existing.insert(item_id, true);
    }

    let atlas_by_item = items
        .iter()
        .filter_map(|entry| {
            let item_id = value_string(entry, "itemId")?;
            Some((item_id, entry.clone()))
        })
        .collect::<BTreeMap<_, _>>();
    let atlas_by_asset = items
        .iter()
        .filter_map(|entry| {
            let asset_id = value_string(entry, "assetId")?;
            Some((asset_id, entry.clone()))
        })
        .collect::<BTreeMap<_, _>>();
    let mut facade_resolutions_by_item = BTreeMap::<String, Vec<&Value>>::new();
    for resolution in facade_resolution_rows {
        if let Some(facade_item_id) = value_string(resolution, "facadeItemId") {
            facade_resolutions_by_item
                .entry(facade_item_id)
                .or_default()
                .push(resolution);
        }
    }
    let mut repaired_facades = 0u64;
    let mut unresolved_facades = Vec::new();
    for item in item_rows {
        let Some(item_id) = value_string(item, "itemId") else {
            continue;
        };
        if existing.contains_key(&item_id) {
            continue;
        }
        if !is_buildcraft_facade(item) {
            continue;
        }
        let facade_asset_id = value_string(item, "renderAssetRef")
            .unwrap_or_else(|| format!("nesqlpp:item/{item_id}"));
        let Some(resolutions) = facade_resolutions_by_item.get(&item_id) else {
            unresolved_facades.push(facade_issue(
                "FACADE_RESOLUTION_FACT_MISSING",
                &item_id,
                &facade_asset_id,
                None,
                "No facadeResolutions row exists for this exact facadeItemId.",
            ));
            continue;
        };
        if resolutions.len() != 1 {
            unresolved_facades.push(facade_issue(
                "FACADE_RESOLUTION_FACT_INVALID",
                &item_id,
                &facade_asset_id,
                None,
                format!(
                    "Expected exactly one facadeResolutions row for facadeItemId, found {}.",
                    resolutions.len()
                ),
            ));
            continue;
        }
        let resolution = resolutions[0];
        let status = value_string(resolution, "status");
        let reason = value_string(resolution, "reason").filter(|value| !value.trim().is_empty());
        let resolved_asset_id =
            value_string(resolution, "facadeAssetId").filter(|value| !value.trim().is_empty());
        if reason.is_none()
            || resolved_asset_id.as_deref() != Some(facade_asset_id.as_str())
            || !matches!(
                status.as_deref(),
                Some("resolved" | "partial" | "unresolved")
            )
        {
            unresolved_facades.push(facade_issue(
                "FACADE_RESOLUTION_FACT_INVALID",
                &item_id,
                &facade_asset_id,
                status.as_deref(),
                "Facade resolution status/reason/asset identity violates the authoritative ABI.",
            ));
            continue;
        }
        let Some(status_value) = status.as_deref() else {
            unreachable!("status was validated above")
        };
        let targets = match validated_facade_targets(resolution, status_value) {
            Ok(targets) => targets,
            Err(error) => {
                unresolved_facades.push(facade_issue(
                    "FACADE_RESOLUTION_FACT_INVALID",
                    &item_id,
                    &facade_asset_id,
                    status.as_deref(),
                    error,
                ));
                continue;
            }
        };
        if status_value == "unresolved" {
            unresolved_facades.push(facade_issue(
                "FACADE_RESOLUTION_UNRESOLVED",
                &item_id,
                &facade_asset_id,
                status.as_deref(),
                reason.unwrap_or_else(|| "Facade resolution is unresolved.".to_string()),
            ));
            continue;
        }
        if targets.is_empty() {
            unresolved_facades.push(facade_issue(
                "FACADE_RESOLUTION_FACT_INVALID",
                &item_id,
                &facade_asset_id,
                status.as_deref(),
                "Resolved or partial facade resolution has no source targets.",
            ));
            continue;
        }
        let Some((resolved_target, source_atlas)) = targets.iter().find_map(|target| {
            if !valid_resolved_facade_target(target) {
                return None;
            }
            let source = exact_facade_source(target, &atlas_by_asset, &atlas_by_item)?;
            (value_string(source, "itemId").as_deref() != Some(item_id.as_str()))
                .then_some((target, source))
        }) else {
            unresolved_facades.push(facade_issue(
                "FACADE_RESOLUTION_SOURCE_MISSING",
                &item_id,
                &facade_asset_id,
                status.as_deref(),
                "No ordered facade target joined to one exact existing sourceAssetId/sourceItemId atlas entry.",
            ));
            continue;
        };
        let mut alias = source_atlas.clone();
        if let Some(object) = alias.as_object_mut() {
            object.insert("itemId".to_string(), json!(item_id));
            object.insert("assetId".to_string(), json!(facade_asset_id));
            object.insert(
                "sourceItemId".to_string(),
                source_atlas.get("itemId").cloned().unwrap_or(Value::Null),
            );
            object.insert(
                "sourceAssetId".to_string(),
                source_atlas.get("assetId").cloned().unwrap_or(Value::Null),
            );
            object.insert(
                "facadeResolutionStatus".to_string(),
                json!(status.unwrap_or_default()),
            );
            object.insert(
                "facadeResolutionReason".to_string(),
                json!(reason.unwrap_or_default()),
            );
            object.insert(
                "facadeResolutionTarget".to_string(),
                resolved_target.clone(),
            );
            object.insert(
                "semanticAtlasAlias".to_string(),
                json!("buildcraft-facade-block-state"),
            );
            object.insert("generatedByCompiler".to_string(), json!(true));
            object.insert(
                "resolutionMode".to_string(),
                json!("authoritative_facade_resolution"),
            );
        }
        items.push(alias);
        existing.insert(item_id, true);
        repaired_facades += 1;
    }

    AtlasRepairResult {
        items,
        repaired_facades,
        unresolved_facades,
    }
}

fn atlas_drawable_score(atlas_entry: Option<&Value>) -> i64 {
    let Some(entry) = atlas_entry else {
        return 0;
    };
    let animated = entry.get("animatedAtlas");
    let static_atlas = entry.get("staticAtlas");
    let animated_file = optional_value_string(animated, "atlasFile");
    let static_file = optional_value_string(static_atlas, "atlasFile");
    let animated_frames = optional_value_u64(animated, "frameCount").unwrap_or(0) as i64;
    let static_width = optional_value_u64(static_atlas, "width").unwrap_or(0) as i64;
    let static_height = optional_value_u64(static_atlas, "height").unwrap_or(0) as i64;
    let mut score = 0i64;
    if animated_file
        .as_deref()
        .unwrap_or_default()
        .trim()
        .is_empty()
        && static_file.as_deref().unwrap_or_default().trim().is_empty()
    {
        return 0;
    }
    if animated_file
        .as_deref()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
        && animated_frames > 0
    {
        score += 10_000 + animated_frames.min(128);
    }
    if static_file
        .as_deref()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
        && static_width > 0
        && static_height > 0
    {
        let area = static_width.saturating_mul(static_height);
        score += 1_000 + area.min(16_384);
        if static_width >= 32 && static_height >= 32 {
            score += 500;
        }
        if static_width >= 64 && static_height >= 64 {
            score += 500;
        }
    }
    if value_string(entry, "resolutionMode")
        .or_else(|| value_string(entry, "renderMode"))
        .map(|mode| mode.contains("captured_final_atlas") || mode.contains("framebuffer"))
        .unwrap_or(false)
    {
        score += 750;
    }
    if value_string(entry, "resolutionMode")
        .or_else(|| value_string(entry, "renderMode"))
        .map(|mode| mode.contains("native_sprite"))
        .unwrap_or(false)
        && static_width <= 16
        && static_height <= 16
    {
        score -= 600;
    }
    score.max(0)
}

pub fn select_group_representative(
    exported_representative: Option<String>,
    members: &[String],
    atlas_by_item: &BTreeMap<String, &Value>,
) -> Option<String> {
    let mut candidates = Vec::new();
    if let Some(representative) = exported_representative.clone() {
        if !representative.trim().is_empty() {
            candidates.push(representative);
        }
    }
    for member in members {
        if !member.trim().is_empty() && !candidates.iter().any(|candidate| candidate == member) {
            candidates.push(member.clone());
        }
    }
    candidates
        .into_iter()
        .enumerate()
        .max_by(|(left_index, left), (right_index, right)| {
            let left_score = atlas_drawable_score(atlas_by_item.get(left).copied());
            let right_score = atlas_drawable_score(atlas_by_item.get(right).copied());
            left_score
                .cmp(&right_score)
                .then_with(|| left_index.cmp(right_index))
        })
        .map(|(_, item_id)| item_id)
        .or(exported_representative)
}
