use crate::atlas_repair::{repaired_browser_atlas_items, select_group_representative};
use crate::binary::{
    intern_compact_string, push_u32, write_binary_pack, write_binary_pack_payload,
};
use crate::io::write_json_value;
use crate::json_ext::{value_string, value_u64};
use crate::manifest::{
    COLLECTION_BROWSER_GROUPS, COLLECTION_BROWSER_ITEMS, COLLECTION_BROWSER_ITEMS_PREFER_CATALOG,
    COLLECTION_FACADE_RESOLUTIONS, COLLECTION_ITEM_IDENTITY_MAP, COLLECTION_NEI_ORDER,
    COLLECTION_SEARCH_ALL_PREFER_INDEX, COLLECTION_SEMANTIC_ITEMS, COLLECTION_TEXTURE_ROWS,
};
use crate::packs::search::{
    build_compact_search_payload_from_items, build_compact_string_payload_from_items,
};
use crate::session::RawExportSession;
use crate::text::{build_pinyin_fields, normalize_search_terms, normalize_text};
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::Path;

pub fn compile_browser_pack(
    session: &RawExportSession,
    output: &Path,
    strict: bool,
    debug_json: bool,
) -> Result<()> {
    let manifest = session.manifest();
    if manifest.files.contains_key("browserCatalog") {
        return compile_dist_browser_pack(session, output, strict, debug_json);
    }
    let items = session.read_manifest_collection(COLLECTION_BROWSER_ITEMS)?;
    let order_rows = session.read_manifest_collection(COLLECTION_NEI_ORDER)?;
    let group_rows = session.read_manifest_collection(COLLECTION_BROWSER_GROUPS)?;
    let semantic_items = session.read_manifest_collection(COLLECTION_SEMANTIC_ITEMS)?;
    let item_identity_map = session.read_manifest_collection(COLLECTION_ITEM_IDENTITY_MAP)?;
    let texture_rows = session.read_manifest_collection(COLLECTION_TEXTURE_ROWS)?;
    let facade_resolutions = session.read_manifest_collection(COLLECTION_FACADE_RESOLUTIONS)?;
    let atlas = session.read_optional_manifest_json("browserAtlasIndex")?;
    let atlas_items = repaired_browser_atlas_items(
        atlas.as_deref().unwrap_or(&Value::Null),
        &texture_rows,
        &items,
        &facade_resolutions,
    )
    .items;

    if strict && items.is_empty() {
        return Err(anyhow!(
            "browser compiler blocked: raw items stream is empty"
        ));
    }

    let order_by_item = order_rows
        .iter()
        .filter_map(|row| {
            Some((
                value_string(row, "itemId")?,
                value_u64(row, "entryOrder").unwrap_or(u64::MAX),
            ))
        })
        .collect::<BTreeMap<_, _>>();

    let atlas_by_item = atlas_items
        .iter()
        .filter_map(|row| Some((value_string(row, "itemId")?, row)))
        .collect::<BTreeMap<_, _>>();

    let raw_groups = group_rows
        .iter()
        .map(|row| {
            let group_key = value_string(row, "groupKey");
            let group_label = value_string(row, "groupLabel");
            let members = row
                .get("memberItemIds")
                .and_then(Value::as_array)
                .map(|values| {
                    values
                        .iter()
                        .filter_map(Value::as_str)
                        .map(str::to_string)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let representative = select_group_representative(
                value_string(row, "representativeItemId"),
                &members,
                &atlas_by_item,
            );
            json!({
                "groupKey": group_key,
                "groupLabel": group_label,
                "groupSize": value_u64(row, "groupSize").unwrap_or(members.len() as u64),
                "groupSortOrder": value_u64(row, "groupSortOrder").unwrap_or(u64::MAX),
                "representativeItemId": representative,
                "memberItemIds": members,
                "groupSource": value_string(row, "groupSource").unwrap_or_else(|| "nativeNei".to_string()),
            })
        })
        .collect::<Vec<_>>();

    let browser_item_ids = collect_browser_item_ids(&order_rows, &group_rows);
    let source_order_by_item = items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| Some((value_string(item, "itemId")?, index as u64)))
        .collect::<BTreeMap<_, _>>();
    let semantic_groups = build_semantic_browser_groups(
        &semantic_items,
        &item_identity_map,
        &browser_item_ids,
        &order_by_item,
        &source_order_by_item,
    );
    let groups = merge_browser_groups(raw_groups, semantic_groups);
    let mut group_by_member = BTreeMap::new();
    for group in &groups {
        let members = group
            .get("memberItemIds")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str);
        for member in members {
            group_by_member.insert(
                member.to_string(),
                json!({
                    "groupKey": value_string(group, "groupKey"),
                    "groupLabel": value_string(group, "groupLabel"),
                    "groupSize": value_u64(group, "groupSize").unwrap_or(1),
                    "representativeItemId": value_string(group, "representativeItemId"),
                    "groupSource": value_string(group, "groupSource"),
                }),
            );
        }
    }
    let browser_source_items = items
        .iter()
        .enumerate()
        .filter(|(_, item)| {
            if browser_item_ids.is_empty() {
                return true;
            }
            value_string(item, "itemId")
                .as_ref()
                .is_some_and(|item_id| browser_item_ids.contains(item_id))
        })
        .collect::<Vec<_>>();
    if strict && !browser_item_ids.is_empty() && browser_source_items.is_empty() {
        return Err(anyhow!(
            "browser compiler blocked: NEI browser contract selected zero items from raw item stream"
        ));
    }

    let semantic_identity_by_item = item_identity_map
        .iter()
        .filter_map(|entry| Some((value_string(entry, "legacyItemId")?, entry)))
        .collect::<BTreeMap<_, _>>();
    let mut alias_map = BTreeMap::new();
    let mut search_items = Vec::new();
    let mut browser_items = browser_source_items
        .iter()
        .map(|item| {
            let (source_order, item) = item;
            let item_id = value_string(item, "itemId").unwrap_or_default();
            let localized_name = value_string(item, "localizedName");
            let mod_id = value_string(item, "modId");
            let internal_name = value_string(item, "internalName");
            let render_asset_ref = value_string(item, "renderAssetRef");
            let semantic_identity = semantic_identity_by_item.get(&item_id).copied();
            let semantic_public_item_id = semantic_identity
                .and_then(|entry| value_string(entry, "publicItemId"));
            let semantic_family = semantic_identity.and_then(|entry| value_string(entry, "family"));
            let semantic_classification =
                semantic_identity.and_then(|entry| value_string(entry, "classification"));
            let semantic_facet_summary =
                semantic_identity.and_then(|entry| value_string(entry, "facetSummary"));
            let (pinyin_full, pinyin_acronym) =
                build_pinyin_fields(localized_name.as_deref().unwrap_or_default());
            let semantic_aliases = semantic_family_aliases(
                semantic_family.as_deref(),
                semantic_classification.as_deref(),
            );
            let facet_aliases = semantic_facet_aliases(semantic_facet_summary.as_deref());
            let group = group_by_member.get(&item_id);
            let group_key = group
                .and_then(|value| value.get("groupKey"))
                .and_then(Value::as_str);
            let group_label = group
                .and_then(|value| value.get("groupLabel"))
                .and_then(Value::as_str);
            let curated_aliases = normalize_search_terms(
                [
                    semantic_family.as_deref(),
                    semantic_classification.as_deref(),
                    (!semantic_aliases.is_empty()).then_some(semantic_aliases.as_str()),
                    (!facet_aliases.is_empty()).then_some(facet_aliases.as_str()),
                    group_key,
                    group_label,
                ]
                .into_iter()
                .flatten(),
            );
            let mut aliases = vec![item_id.clone()];
            for value in [
                localized_name.clone(),
                mod_id.clone(),
                internal_name.clone(),
                render_asset_ref.clone(),
                (!curated_aliases.is_empty()).then_some(curated_aliases.clone()),
            ]
            .into_iter()
            .flatten()
            {
                if !value.trim().is_empty() {
                    aliases.push(value);
                }
            }
            aliases.sort();
            aliases.dedup();
            alias_map.insert(item_id.clone(), aliases);
            let public_item_id = semantic_public_item_id
                .unwrap_or_else(|| format!("item:{}", item_id.to_ascii_lowercase()));
            let browser_order = order_by_item
                .get(&item_id)
                .copied()
                .unwrap_or(*source_order as u64);
            let normalized_terms = normalize_search_terms(
                [
                    localized_name.as_deref(),
                    internal_name.as_deref(),
                    mod_id.as_deref(),
                    Some(public_item_id.as_str()),
                    semantic_family.as_deref(),
                    semantic_classification.as_deref(),
                    semantic_facet_summary.as_deref(),
                    (!semantic_aliases.is_empty()).then_some(semantic_aliases.as_str()),
                    (!facet_aliases.is_empty()).then_some(facet_aliases.as_str()),
                    group_key,
                    group_label,
                ]
                .into_iter()
                .flatten(),
            );
            search_items.push(json!({
                "itemId": item_id,
                "publicItemId": public_item_id,
                "localizedName": localized_name,
                "modId": mod_id,
                "internalName": internal_name,
                "normalizedLocalizedName": localized_name.as_deref().map(normalize_text).unwrap_or_default(),
                "normalizedInternalName": internal_name.as_deref().map(normalize_text).unwrap_or_default(),
                "normalizedItemId": normalize_text(&item_id),
                "normalizedSearchTerms": normalized_terms,
                "pinyinFull": pinyin_full,
                "pinyinAcronym": pinyin_acronym,
                "aliases": curated_aliases,
                "popularityScore": group.and_then(|value| value.get("groupSize")).and_then(Value::as_u64).unwrap_or(1),
                "searchRank": browser_order,
                "renderAssetRef": render_asset_ref,
                "family": semantic_family,
                "classification": semantic_classification,
                "facetSummary": semantic_facet_summary,
                "groupKey": group.and_then(|value| value.get("groupKey")).cloned().unwrap_or(Value::Null),
                "groupLabel": group.and_then(|value| value.get("groupLabel")).cloned().unwrap_or(Value::Null),
                "groupSize": group.and_then(|value| value.get("groupSize")).cloned().unwrap_or(json!(1)),
                "representativeItemId": group
                    .and_then(|value| value.get("representativeItemId"))
                    .cloned()
                    .unwrap_or_else(|| json!(item_id)),
            }));
            json!({
                "itemId": item_id,
                "publicItemId": public_item_id,
                "localizedName": localized_name,
                "modId": mod_id,
                "internalName": internal_name,
                "renderAssetRef": render_asset_ref,
                "browserOrder": browser_order,
                "semanticFamily": semantic_family,
                "semanticClassification": semantic_classification,
                "facetSummary": semantic_facet_summary,
                "groupKey": group.and_then(|value| value.get("groupKey")).cloned().unwrap_or(Value::Null),
                "groupLabel": group.and_then(|value| value.get("groupLabel")).cloned().unwrap_or(Value::Null),
                "groupSize": group.and_then(|value| value.get("groupSize")).cloned().unwrap_or(json!(1)),
                "representativeItemId": group
                    .and_then(|value| value.get("representativeItemId"))
                    .cloned()
                    .unwrap_or_else(|| json!(item_id)),
                "groupSource": group.and_then(|value| value.get("groupSource")).cloned().unwrap_or(Value::Null),
                "atlas": atlas_by_item
                    .get(&item_id)
                    .map(|value| (*value).clone())
                    .unwrap_or(Value::Null),
            })
        })
        .collect::<Vec<_>>();

    browser_items.sort_by(|left, right| {
        let left_order = left
            .get("browserOrder")
            .and_then(Value::as_u64)
            .unwrap_or(u64::MAX);
        let right_order = right
            .get("browserOrder")
            .and_then(Value::as_u64)
            .unwrap_or(u64::MAX);
        left_order
            .cmp(&right_order)
            .then_with(|| value_string(left, "itemId").cmp(&value_string(right, "itemId")))
    });
    let browser_index_by_item = browser_items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| Some((value_string(item, "itemId")?, index as u64)))
        .collect::<BTreeMap<_, _>>();
    for item in &mut search_items {
        if let Some(item_object) = item.as_object_mut() {
            if let Some(item_id) = item_object.get("itemId").and_then(Value::as_str) {
                if let Some(browser_index) = browser_index_by_item.get(item_id) {
                    item_object.insert("browserIndex".to_string(), json!(browser_index));
                }
            }
        }
    }

    let atlas_items = atlas_by_item.len() as u64;
    let pack = json!({
        "schemaVersion": "neonei/rust-browser-pack/current",
        "counts": {
            "items": browser_items.len(),
            "rawItems": items.len(),
            "groups": groups.len(),
            "aliasItems": alias_map.len(),
            "orderedItems": order_by_item.len(),
            "browserContractItems": browser_item_ids.len(),
            "atlasItems": atlas_items,
            "missingAtlas": browser_items.iter().filter(|item| item.get("atlas") == Some(&Value::Null)).count(),
        },
        "items": browser_items,
        "groups": groups,
        "aliasMap": alias_map,
    });

    let rust_dir = output.join("rust");
    fs::create_dir_all(&rust_dir)?;
    if debug_json {
        write_json_value(&rust_dir.join("browser-pack.json"), &pack)?;
    }
    let compact_browser_payload = build_compact_browser_payload_from_items(&browser_items)?;
    write_binary_pack_payload(
        &rust_dir.join("browser.bin"),
        "neonei/browser-pack/current",
        &compact_browser_payload,
    )?;
    let search_pack = json!({
        "schemaVersion": "neonei/rust-search-pack/current",
        "counts": {
            "items": search_items.len(),
            "aliasItems": alias_map.len(),
        },
        "items": search_items,
    });
    let string_pack = build_compact_string_payload_from_items(&browser_items)?;
    let compact_search_payload = build_compact_search_payload_from_items(
        search_pack
            .get("items")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or(&[]),
    )?;
    if debug_json {
        write_json_value(&rust_dir.join("search-pack.json"), &search_pack)?;
    }
    write_binary_pack_payload(
        &rust_dir.join("search.bin"),
        "neonei/search-pack/current",
        &compact_search_payload,
    )?;
    let group_payload = build_compact_group_payload_from_groups(&groups)?;
    write_binary_pack_payload(
        &rust_dir.join("groups.bin"),
        "neonei/group-pack/current",
        &group_payload,
    )?;
    write_binary_pack_payload(
        &rust_dir.join("strings.zh_cn.bin"),
        "neonei/string-pack/current",
        &string_pack,
    )?;
    Ok(())
}

fn collect_browser_item_ids(order_rows: &[Value], group_rows: &[Value]) -> HashSet<String> {
    let mut ids = HashSet::new();
    for row in order_rows {
        if let Some(item_id) = value_string(row, "itemId") {
            ids.insert(item_id);
        }
    }
    for row in group_rows {
        if let Some(representative) = value_string(row, "representativeItemId") {
            ids.insert(representative);
        }
        if let Some(members) = row.get("memberItemIds").and_then(Value::as_array) {
            for member in members {
                if let Some(item_id) = member.as_str() {
                    ids.insert(item_id.to_string());
                }
            }
        }
    }
    ids
}

fn build_semantic_browser_groups(
    semantic_items: &[Value],
    item_identity_map: &[Value],
    available_item_ids: &HashSet<String>,
    order_by_item: &BTreeMap<String, u64>,
    source_order_by_item: &BTreeMap<String, u64>,
) -> Vec<Value> {
    let semantic_by_public_id = semantic_items
        .iter()
        .filter_map(|item| Some((value_string(item, "publicItemId")?, item)))
        .collect::<BTreeMap<_, _>>();
    let mut members_by_public_id = BTreeMap::<String, Vec<(u64, String, &Value)>>::new();

    for entry in item_identity_map {
        let Some(public_item_id) = value_string(entry, "publicItemId") else {
            continue;
        };
        let Some(legacy_item_id) = value_string(entry, "legacyItemId") else {
            continue;
        };
        if !available_item_ids.contains(&legacy_item_id) {
            continue;
        }
        let browser_order = order_by_item
            .get(&legacy_item_id)
            .or_else(|| source_order_by_item.get(&legacy_item_id))
            .copied()
            .unwrap_or(u64::MAX);
        members_by_public_id
            .entry(public_item_id)
            .or_default()
            .push((browser_order, legacy_item_id, entry));
    }

    let mut groups = Vec::new();
    for (public_item_id, mut members) in members_by_public_id {
        members.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
        members.dedup_by(|left, right| left.1 == right.1);
        if members.len() <= 1 {
            continue;
        }

        let semantic_item = semantic_by_public_id.get(&public_item_id).copied();
        let member_item_ids = members
            .iter()
            .map(|(_, item_id, _)| item_id.clone())
            .collect::<Vec<_>>();
        let declared_representative =
            semantic_item.and_then(|item| value_string(item, "representativeLegacyItemId"));
        let representative_item_id = declared_representative
            .filter(|item_id| member_item_ids.contains(item_id))
            .unwrap_or_else(|| member_item_ids[0].clone());
        let group_key = public_item_id
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>();
        let group_label = semantic_item
            .and_then(|item| {
                value_string(item, "localizedName").or_else(|| value_string(item, "internalName"))
            })
            .unwrap_or_else(|| group_key.clone());
        let first_identity = members[0].2;
        let semantic_family = semantic_item
            .and_then(|item| value_string(item, "family"))
            .or_else(|| value_string(first_identity, "family"));
        let semantic_classification = semantic_item
            .and_then(|item| value_string(item, "classification"))
            .or_else(|| value_string(first_identity, "classification"));

        groups.push(json!({
            "groupKey": group_key,
            "groupLabel": group_label,
            "groupSize": member_item_ids.len(),
            "representativeItemId": representative_item_id,
            "memberItemIds": member_item_ids,
            "groupSortOrder": members[0].0,
            "groupSource": "semanticIdentity",
            "publicItemId": public_item_id,
            "semanticFamily": semantic_family,
            "semanticClassification": semantic_classification,
        }));
    }
    groups.sort_by(|left, right| {
        value_u64(left, "groupSortOrder")
            .unwrap_or(0)
            .cmp(&value_u64(right, "groupSortOrder").unwrap_or(0))
            .then_with(|| value_string(left, "groupKey").cmp(&value_string(right, "groupKey")))
    });
    groups
}

fn merge_browser_groups(raw_groups: Vec<Value>, semantic_groups: Vec<Value>) -> Vec<Value> {
    let mut candidates = raw_groups
        .into_iter()
        .map(|mut group| {
            if group.get("groupSource").and_then(Value::as_str).is_none() {
                let source = infer_browser_group_source(&group);
                if let Some(object) = group.as_object_mut() {
                    object.insert("groupSource".to_string(), json!(source));
                }
            }
            group
        })
        .chain(semantic_groups)
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        browser_group_precedence(left)
            .cmp(&browser_group_precedence(right))
            .then_with(|| {
                value_u64(left, "groupSortOrder")
                    .unwrap_or(0)
                    .cmp(&value_u64(right, "groupSortOrder").unwrap_or(0))
            })
            .then_with(|| value_string(left, "groupKey").cmp(&value_string(right, "groupKey")))
    });

    let mut assigned = HashSet::new();
    let mut merged = Vec::new();
    for mut group in candidates {
        let mut original_members = group
            .get("memberItemIds")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect::<Vec<_>>();
        let mut seen = HashSet::new();
        original_members.retain(|item_id| seen.insert(item_id.clone()));
        let retained_members = original_members
            .iter()
            .filter(|item_id| !assigned.contains(*item_id))
            .cloned()
            .collect::<Vec<_>>();
        let source = value_string(&group, "groupSource").unwrap_or_default();
        let authoritative_raw_group = source == "rawExport"
            || source == "collapsibleItems"
            || browser_group_precedence(&group) <= 20;

        let members = if retained_members.len() <= 1 {
            if authoritative_raw_group && !original_members.is_empty() {
                original_members
            } else {
                continue;
            }
        } else {
            for item_id in &retained_members {
                assigned.insert(item_id.clone());
            }
            retained_members
        };
        let representative_item_id = value_string(&group, "representativeItemId")
            .filter(|item_id| members.contains(item_id))
            .unwrap_or_else(|| members[0].clone());
        let group_size = members.len();
        if let Some(object) = group.as_object_mut() {
            object.insert("memberItemIds".to_string(), json!(members));
            object.insert("groupSize".to_string(), json!(group_size));
            object.insert(
                "representativeItemId".to_string(),
                json!(representative_item_id),
            );
        }
        merged.push(group);
    }
    merged
}

fn browser_group_precedence(group: &Value) -> u8 {
    let group_key = value_string(group, "groupKey").unwrap_or_default();
    let source = value_string(group, "groupSource").unwrap_or_default();
    if group_key.starts_with("nei:") || source == "nativeNei" || source == "collapsibleItems" {
        10
    } else if source == "guidfilters" || group_key.starts_with("guidfilter:") {
        20
    } else if source == "semanticIdentity" || group_key.starts_with("semantic:") {
        30
    } else if source == "syntheticFallback"
        || source == "fallback"
        || group_key.starts_with("fallback:")
    {
        40
    } else {
        35
    }
}

fn infer_browser_group_source(group: &Value) -> String {
    let group_key = value_string(group, "groupKey").unwrap_or_default();
    if group_key.starts_with("fallback:") {
        "fallback"
    } else if group_key.starts_with("semantic:") {
        "semanticIdentity"
    } else if group_key.starts_with("guidfilter:") {
        "guidfilters"
    } else if group_key.starts_with("nei:") {
        "nativeNei"
    } else if group_key.starts_with("variant:") {
        "syntheticFallback"
    } else {
        "rawExport"
    }
    .to_string()
}

fn semantic_family_aliases(family: Option<&str>, classification: Option<&str>) -> String {
    let normalized = family.unwrap_or_default().trim().to_ascii_lowercase();
    let mut aliases = Vec::new();
    if normalized.starts_with("facade.") {
        aliases.extend([
            "facade",
            "cover",
            "camouflage",
            "microblock",
            "painted block",
        ]);
    }
    match normalized.as_str() {
        "facade.buildcraft" => aliases.extend(["buildcraft facade", "pipe facade"]),
        "facade.ae2" => aliases.extend(["ae2 facade", "applied energistics facade"]),
        "facade.enderio.paint" => aliases.extend(["enderio painted block", "conduit facade"]),
        "thaumcraft.wand" => aliases.extend([
            "wand",
            "sceptre",
            "scepter",
            "staff",
            "focus",
            "rod",
            "cap",
            "thaumcraft wand",
        ]),
        "tool.tconstruct" => aliases.extend([
            "tinkers tool",
            "tconstruct tool",
            "infitool",
            "modifier",
            "durability",
        ]),
        "toolpart.tconstruct" => aliases.extend([
            "tinkers part",
            "tconstruct part",
            "tool part",
            "bolt part",
            "arrow part",
        ]),
        "toolpart.tgregworks" => aliases.extend([
            "tgregworks part",
            "gregworks part",
            "tool part",
            "material part",
        ]),
        "tool.gregtech" => {
            aliases.extend(["gregtech tool", "gt tool", "meta tool", "electric tool"])
        }
        "crop.ic2" => aliases.extend(["ic2 crop", "crop seed", "growth", "gain", "resistance"]),
        "fluid.container" => aliases.extend([
            "fluid cell",
            "fluid container",
            "bucket",
            "capsule",
            "tank",
            "fluid",
        ]),
        "data_carrier.encoded-pattern" => aliases.extend([
            "encoded pattern",
            "ae2 pattern",
            "processing pattern",
            "crafting pattern",
        ]),
        "cosmetic.color" => aliases.extend(["color", "colour", "dye", "painted", "cosmetic"]),
        _ => {}
    }
    if normalized.starts_with("genetics.") {
        aliases.extend([
            "bee",
            "tree",
            "butterfly",
            "genetics",
            "genome",
            "allele",
            "species",
            "serum",
            "template",
        ]);
    }
    if normalized.starts_with("entity_capture.") {
        aliases.extend([
            "mob soul",
            "mob crystal",
            "soul vial",
            "entity capture",
            "monster",
            "mob",
        ]);
    }
    if let Some(classification) = classification
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        aliases.push(classification);
    }
    let mut seen = HashSet::new();
    aliases
        .into_iter()
        .filter(|alias| seen.insert(*alias))
        .collect::<Vec<_>>()
        .join(" ")
}

fn semantic_facet_aliases(facet_summary: Option<&str>) -> String {
    facet_summary
        .unwrap_or_default()
        .split(|character: char| {
            character.is_whitespace()
                || matches!(character, '=' | ':' | ',' | ';' | '|' | '/' | '\\')
        })
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn build_compact_browser_payload(session: &RawExportSession) -> Result<Vec<u8>> {
    let items = session.read_manifest_collection(COLLECTION_BROWSER_ITEMS_PREFER_CATALOG)?;
    build_compact_browser_payload_from_items(&items)
}

pub fn build_compact_browser_payload_from_items(items: &[Value]) -> Result<Vec<u8>> {
    let mut strings = vec![String::new()];
    let mut string_refs = HashMap::new();
    string_refs.insert(String::new(), 0u32);
    let mut rows = Vec::<[u32; 6]>::with_capacity(items.len());

    for (index, item) in items.iter().enumerate() {
        let item_id_ref =
            intern_compact_string(&mut strings, &mut string_refs, value_string(item, "itemId"));
        let localized_name_ref = intern_compact_string(
            &mut strings,
            &mut string_refs,
            value_string(item, "localizedName"),
        );
        let mod_id_ref =
            intern_compact_string(&mut strings, &mut string_refs, value_string(item, "modId"));
        let group_key_ref = intern_compact_string(
            &mut strings,
            &mut string_refs,
            value_string(item, "groupKey"),
        );
        let browser_order = value_u64(item, "browserOrder")
            .unwrap_or(index as u64)
            .min(u32::MAX as u64) as u32;
        let flags = if item
            .get("groupKey")
            .and_then(Value::as_str)
            .is_some_and(|value| !value.is_empty())
        {
            1
        } else {
            0
        };
        rows.push([
            item_id_ref,
            localized_name_ref,
            mod_id_ref,
            group_key_ref,
            browser_order,
            flags,
        ]);
    }

    let mut string_offsets = Vec::<u32>::with_capacity(strings.len());
    let mut string_bytes = Vec::<u8>::new();
    for value in &strings {
        string_offsets.push(string_bytes.len() as u32);
        string_bytes.extend_from_slice(value.as_bytes());
        string_bytes.push(0);
    }

    let row_stride_u32 = 6u32;
    let mut payload = Vec::with_capacity(
        8 + 4 * 4
            + string_offsets.len() * 4
            + rows.len() * row_stride_u32 as usize * 4
            + string_bytes.len(),
    );
    payload.extend_from_slice(b"NEIBRW1\0");
    push_u32(&mut payload, 1);
    push_u32(&mut payload, rows.len() as u32);
    push_u32(&mut payload, strings.len() as u32);
    push_u32(&mut payload, row_stride_u32);
    for offset in string_offsets {
        push_u32(&mut payload, offset);
    }
    for row in rows {
        for value in row {
            push_u32(&mut payload, value);
        }
    }
    payload.extend_from_slice(&string_bytes);
    Ok(payload)
}

pub fn build_compact_group_payload_from_groups(groups: &[Value]) -> Result<Vec<u8>> {
    let mut strings = vec![String::new()];
    let mut string_refs = HashMap::new();
    string_refs.insert(String::new(), 0u32);
    let mut rows = Vec::<[u32; 6]>::new();
    let mut members = Vec::<u32>::new();

    let mut sorted_groups = groups.to_vec();
    sorted_groups.sort_by(|left, right| {
        value_string(left, "groupKey").cmp(&value_string(right, "groupKey"))
    });

    for group in &sorted_groups {
        let group_key = intern_compact_string(
            &mut strings,
            &mut string_refs,
            value_string(group, "groupKey"),
        );
        let group_label = intern_compact_string(
            &mut strings,
            &mut string_refs,
            value_string(group, "groupLabel"),
        );
        let representative = intern_compact_string(
            &mut strings,
            &mut string_refs,
            value_string(group, "representativeItemId"),
        );
        let member_start = members.len() as u32;
        if let Some(values) = group.get("memberItemIds").and_then(Value::as_array) {
            for member in values {
                members.push(intern_compact_string(
                    &mut strings,
                    &mut string_refs,
                    member.as_str().map(str::to_string),
                ));
            }
        }
        let member_count = (members.len() as u32).saturating_sub(member_start);
        rows.push([
            group_key,
            group_label,
            representative,
            member_start,
            member_count,
            value_u64(group, "groupSize").unwrap_or(member_count as u64) as u32,
        ]);
    }

    let mut string_offsets = Vec::<u32>::with_capacity(strings.len());
    let mut string_bytes = Vec::<u8>::new();
    for value in &strings {
        string_offsets.push(string_bytes.len() as u32);
        string_bytes.extend_from_slice(value.as_bytes());
        string_bytes.push(0);
    }

    let row_stride_u32 = 6u32;
    let mut payload = Vec::with_capacity(
        8 + 5 * 4
            + string_offsets.len() * 4
            + rows.len() * row_stride_u32 as usize * 4
            + members.len() * 4
            + string_bytes.len(),
    );
    payload.extend_from_slice(b"NEIGRP1\0");
    push_u32(&mut payload, 1);
    push_u32(&mut payload, rows.len() as u32);
    push_u32(&mut payload, strings.len() as u32);
    push_u32(&mut payload, members.len() as u32);
    push_u32(&mut payload, row_stride_u32);
    for offset in string_offsets {
        push_u32(&mut payload, offset);
    }
    for row in rows {
        for value in row {
            push_u32(&mut payload, value);
        }
    }
    for member in members {
        push_u32(&mut payload, member);
    }
    payload.extend_from_slice(&string_bytes);
    Ok(payload)
}

pub fn compile_dist_browser_pack(
    session: &RawExportSession,
    output: &Path,
    strict: bool,
    debug_json: bool,
) -> Result<()> {
    let browser_files = session.runtime_file_descriptors(&[
        ("browserCatalog", "browserCatalog"),
        ("hiddenBrowserCatalog", "hiddenBrowserCatalog"),
        ("groups", "browserGroups"),
        ("nativeNeiRules", "nativeNeiRules"),
        ("searchAll", "searchAll"),
        ("searchAliasIndex", "searchAliasIndex"),
    ])?;
    if strict
        && !browser_files.iter().any(|value| {
            value
                .get("logicalName")
                .and_then(Value::as_str)
                .is_some_and(|value| value == "browserCatalog")
        })
    {
        return Err(anyhow!(
            "browser compiler blocked: browserCatalog is missing"
        ));
    }

    let browser_pack = json!({
        "schemaVersion": "neonei/rust-browser-pack/current",
        "sourceKind": "dist-data",
        "counts": { "files": browser_files.len() },
        "files": browser_files,
    });
    let compact_browser_payload = build_compact_browser_payload(session)?;
    let group_pack = json!({
        "schemaVersion": "neonei/rust-group-pack/current",
        "sourceKind": "dist-data",
        "files": session.runtime_file_descriptors(&[("groups", "browserGroups")])?,
    });
    let search_pack = json!({
        "schemaVersion": "neonei/rust-search-pack/current",
        "sourceKind": "dist-data",
        "files": session.runtime_file_descriptors(
            &[("searchAll", "searchAll"), ("searchAliasIndex", "searchAliasIndex")],
        )?,
    });

    let rust_dir = output.join("rust");
    fs::create_dir_all(&rust_dir)?;
    if debug_json {
        write_json_value(&rust_dir.join("browser-pack.json"), &browser_pack)?;
    }
    if debug_json {
        write_json_value(&rust_dir.join("search-pack.json"), &search_pack)?;
    }
    write_binary_pack_payload(
        &rust_dir.join("browser.bin"),
        "neonei/browser-pack/current",
        &compact_browser_payload,
    )?;
    write_binary_pack(
        &rust_dir.join("groups.bin"),
        "neonei/group-pack/current",
        &group_pack,
    )?;
    let browser_items =
        session.read_manifest_collection(COLLECTION_BROWSER_ITEMS_PREFER_CATALOG)?;
    let browser_index_by_item = browser_items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| Some((value_string(item, "itemId")?, index as u64)))
        .collect::<BTreeMap<_, _>>();
    let mut search_rows = session
        .read_manifest_collection(COLLECTION_SEARCH_ALL_PREFER_INDEX)?
        .as_ref()
        .clone();
    for item in &mut search_rows {
        if let Some(item_object) = item.as_object_mut() {
            if let Some(item_id) = item_object.get("itemId").and_then(Value::as_str) {
                if let Some(browser_index) = browser_index_by_item.get(item_id) {
                    item_object.insert("browserIndex".to_string(), json!(browser_index));
                }
            }
        }
    }
    let compact_search_payload = build_compact_search_payload_from_items(&search_rows)?;
    write_binary_pack_payload(
        &rust_dir.join("search.bin"),
        "neonei/search-pack/current",
        &compact_search_payload,
    )?;
    let string_pack = build_compact_string_payload_from_items(&browser_items)?;
    write_binary_pack_payload(
        &rust_dir.join("strings.zh_cn.bin"),
        "neonei/string-pack/current",
        &string_pack,
    )?;
    Ok(())
}
