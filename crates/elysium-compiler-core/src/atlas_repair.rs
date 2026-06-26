use crate::json_ext::{optional_value_string, optional_value_u64, value_string};
use serde_json::{json, Value};
use std::collections::BTreeMap;

fn item_id_from_asset_id(asset_id: &str) -> Option<String> {
    asset_id.strip_prefix("nesqlpp:item/").map(str::to_string)
}

fn normalize_block_lookup_key(mod_id: &str, internal_name: &str, damage: u64) -> Option<String> {
    let normalized_mod = mod_id
        .trim()
        .to_ascii_lowercase()
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
        .collect::<String>();
    let normalized_name = internal_name.trim().to_ascii_lowercase();
    if normalized_mod.is_empty() || normalized_name.is_empty() {
        None
    } else {
        Some(format!("{}:{}:{}", normalized_mod, normalized_name, damage))
    }
}

fn decode_item_block_key(item_id: &str) -> Option<String> {
    let parts = item_id.split('~').collect::<Vec<_>>();
    if parts.len() < 4 {
        return None;
    }
    let damage = parts[3].parse::<u64>().unwrap_or(0);
    normalize_block_lookup_key(parts[1], parts[2], damage)
}

fn parse_buildcraft_facade_target(item: &Value) -> Option<(String, String, u64)> {
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
    let is_facade = family == "facade.buildcraft"
        || (item_mod_id == "buildcrafttransport" && item_internal_name == "pipefacade");
    if !is_facade {
        return None;
    }
    let descriptor = value_string(item, "nbtDescriptor")?;
    let block_marker = "block:";
    let block_start = descriptor.find(block_marker)? + block_marker.len();
    let block_tail = descriptor[block_start..].trim_start();
    let block_tail = block_tail.strip_prefix('"').unwrap_or(block_tail);
    let block_end = block_tail
        .find(|ch| ch == '"' || ch == ',' || ch == '}')
        .unwrap_or(block_tail.len());
    let block = &block_tail[..block_end];
    let (mod_id, internal_name) = block.split_once(':')?;
    let damage = descriptor
        .find("metadata:")
        .and_then(|index| {
            let tail = descriptor[index + "metadata:".len()..].trim_start();
            let digits = tail
                .chars()
                .take_while(|ch| ch.is_ascii_digit() || *ch == '-')
                .collect::<String>();
            digits.parse::<i64>().ok()
        })
        .unwrap_or(0)
        .max(0) as u64;
    Some((mod_id.to_string(), internal_name.to_string(), damage))
}

pub fn repaired_browser_atlas(atlas: &Value, texture_rows: &[Value], item_rows: &[Value]) -> Value {
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

    let atlas_by_block = items
        .iter()
        .filter_map(|entry| {
            let item_id = value_string(entry, "itemId")?;
            let key = decode_item_block_key(&item_id)?;
            Some((key, entry.clone()))
        })
        .collect::<BTreeMap<_, _>>();
    let mut repaired_facades = 0u64;
    for item in item_rows {
        let Some(item_id) = value_string(item, "itemId") else {
            continue;
        };
        if existing.contains_key(&item_id) {
            continue;
        }
        let Some((mod_id, internal_name, damage)) = parse_buildcraft_facade_target(item) else {
            continue;
        };
        let Some(key) = normalize_block_lookup_key(&mod_id, &internal_name, damage) else {
            continue;
        };
        let Some(source_atlas) = atlas_by_block.get(&key) else {
            continue;
        };
        let mut alias = source_atlas.clone();
        if let Some(object) = alias.as_object_mut() {
            object.insert("itemId".to_string(), json!(item_id));
            object.insert(
                "assetId".to_string(),
                json!(value_string(item, "renderAssetRef").unwrap_or_else(|| {
                    format!(
                        "nesqlpp:item/{}",
                        value_string(item, "itemId").unwrap_or_default()
                    )
                })),
            );
            object.insert(
                "sourceItemId".to_string(),
                source_atlas
                    .get("itemId")
                    .cloned()
                    .unwrap_or_else(|| Value::Null),
            );
            object.insert(
                "semanticAtlasAlias".to_string(),
                json!("buildcraft-facade-block-state"),
            );
            object.insert("generatedByCompiler".to_string(), json!(true));
            object.insert(
                "resolutionMode".to_string(),
                json!("rust_buildcraft_facade_alias"),
            );
        }
        items.push(alias);
        existing.insert(item_id, true);
        repaired_facades += 1;
    }

    let mut repaired = atlas.clone();
    if !repaired.is_object() {
        repaired = json!({ "schemaVersion": "browser-atlas-index-repaired" });
    }
    if let Some(object) = repaired.as_object_mut() {
        object.insert("items".to_string(), Value::Array(items));
        let item_count = object
            .get("items")
            .and_then(Value::as_array)
            .map(|values| values.len() as u64)
            .unwrap_or(0);
        object.insert("itemCount".to_string(), json!(item_count));
        object.insert(
            "rustRepairedBuildCraftFacades".to_string(),
            json!(repaired_facades),
        );
    }
    repaired
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
    atlas_by_item: &BTreeMap<String, Value>,
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
            let left_score = atlas_drawable_score(atlas_by_item.get(left));
            let right_score = atlas_drawable_score(atlas_by_item.get(right));
            left_score
                .cmp(&right_score)
                .then_with(|| left_index.cmp(right_index))
        })
        .map(|(_, item_id)| item_id)
        .or(exported_representative)
}
