use crate::atlas_repair::{repaired_browser_atlas, select_group_representative};
use crate::binary::{
    intern_compact_string, push_u32, write_binary_pack, write_binary_pack_payload,
};
use crate::io::write_json_value;
use crate::json_ext::{value_string, value_u64};
use crate::manifest::{
    read_json_collection, read_manifest, read_optional_manifest_json, runtime_file_descriptors,
    RawManifest,
};
use crate::packs::search::{
    build_compact_search_payload_from_items, build_compact_string_payload_from_items,
};
use crate::text::{build_pinyin_fields, normalize_search_terms, normalize_text};
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

pub fn compile_browser_pack(
    input: &Path,
    output: &Path,
    strict: bool,
    debug_json: bool,
) -> Result<()> {
    let manifest = read_manifest(input)?;
    if manifest.files.contains_key("browserCatalog") {
        return compile_dist_browser_pack(input, output, strict, debug_json);
    }
    let items = read_json_collection(
        input,
        &manifest,
        &["items", "browserCatalog"],
        Some("items"),
    )?;
    let order_rows = read_json_collection(input, &manifest, &["neiOrder"], None)?;
    let group_rows = read_json_collection(
        input,
        &manifest,
        &["groups", "browserGroups"],
        Some("groups"),
    )?;
    let texture_rows = read_json_collection(input, &manifest, &["textures"], Some("textures"))?;
    let atlas =
        read_optional_manifest_json(input, &manifest, "browserAtlasIndex")?.unwrap_or(Value::Null);
    let atlas = repaired_browser_atlas(&atlas, &texture_rows, &items);

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

    let atlas_by_item = atlas
        .get("items")
        .and_then(Value::as_array)
        .map(|rows| {
            rows.iter()
                .filter_map(|row| Some((value_string(row, "itemId")?, row.clone())))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();

    let mut group_by_member = BTreeMap::new();
    let groups = group_rows
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
            for member in &members {
                group_by_member.insert(
                    member.clone(),
                    json!({
                        "groupKey": group_key,
                        "groupLabel": group_label,
                        "groupSize": value_u64(row, "groupSize").unwrap_or(members.len() as u64),
                        "representativeItemId": representative,
                        "groupSource": "nativeNei",
                    }),
                );
            }
            json!({
                "groupKey": group_key,
                "groupLabel": group_label,
                "groupSize": value_u64(row, "groupSize").unwrap_or(members.len() as u64),
                "representativeItemId": representative,
                "memberItemIds": members,
                "groupSource": "nativeNei",
            })
        })
        .collect::<Vec<_>>();

    let mut alias_map = BTreeMap::new();
    let mut search_items = Vec::new();
    let mut browser_items = items
        .iter()
        .enumerate()
        .map(|item| {
            let (search_rank, item) = item;
            let item_id = value_string(item, "itemId").unwrap_or_default();
            let localized_name = value_string(item, "localizedName");
            let mod_id = value_string(item, "modId");
            let internal_name = value_string(item, "internalName");
            let render_asset_ref = value_string(item, "renderAssetRef");
            let raw_search_terms = value_string(item, "searchTerms");
            let (pinyin_full, pinyin_acronym) =
                build_pinyin_fields(localized_name.as_deref().unwrap_or_default());
            let mut aliases = vec![item_id.clone()];
            for value in [
                localized_name.clone(),
                mod_id.clone(),
                internal_name.clone(),
                render_asset_ref.clone(),
                raw_search_terms.clone(),
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
            let group = group_by_member.get(&item_id);
            let public_item_id = format!("item:{}", item_id.to_ascii_lowercase());
            let normalized_terms = normalize_search_terms(
                [
                    localized_name.as_deref(),
                    internal_name.as_deref(),
                    mod_id.as_deref(),
                    raw_search_terms.as_deref(),
                    Some(public_item_id.as_str()),
                    group
                        .and_then(|value| value.get("groupKey"))
                        .and_then(Value::as_str),
                    group
                        .and_then(|value| value.get("groupLabel"))
                        .and_then(Value::as_str),
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
                "aliases": raw_search_terms.unwrap_or_default(),
                "popularityScore": group.and_then(|value| value.get("groupSize")).and_then(Value::as_u64).unwrap_or(1),
                "searchRank": search_rank,
                "renderAssetRef": render_asset_ref,
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
                "browserOrder": order_by_item.get(&item_id).copied().unwrap_or(u64::MAX),
                "groupKey": group.and_then(|value| value.get("groupKey")).cloned().unwrap_or(Value::Null),
                "groupLabel": group.and_then(|value| value.get("groupLabel")).cloned().unwrap_or(Value::Null),
                "groupSize": group.and_then(|value| value.get("groupSize")).cloned().unwrap_or(json!(1)),
                "representativeItemId": group
                    .and_then(|value| value.get("representativeItemId"))
                    .cloned()
                    .unwrap_or_else(|| json!(item_id)),
                "groupSource": group.and_then(|value| value.get("groupSource")).cloned().unwrap_or(Value::Null),
                "atlas": atlas_by_item.get(&item_id).cloned().unwrap_or(Value::Null),
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
            "groups": groups.len(),
            "aliasItems": alias_map.len(),
            "orderedItems": order_by_item.len(),
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

fn build_compact_browser_payload(input: &Path, manifest: &RawManifest) -> Result<Vec<u8>> {
    let items = read_json_collection(input, manifest, &["browserCatalog", "items"], Some("items"))?;
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
    input: &Path,
    output: &Path,
    strict: bool,
    debug_json: bool,
) -> Result<()> {
    let manifest = read_manifest(input)?;
    let browser_files = runtime_file_descriptors(
        input,
        &manifest,
        &[
            ("browserCatalog", "browserCatalog"),
            ("hiddenBrowserCatalog", "hiddenBrowserCatalog"),
            ("groups", "browserGroups"),
            ("nativeNeiRules", "nativeNeiRules"),
            ("searchAll", "searchAll"),
            ("searchAliasIndex", "searchAliasIndex"),
        ],
    )?;
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
    let compact_browser_payload = build_compact_browser_payload(input, &manifest)?;
    let group_pack = json!({
        "schemaVersion": "neonei/rust-group-pack/current",
        "sourceKind": "dist-data",
        "files": runtime_file_descriptors(input, &manifest, &[("groups", "browserGroups")])?,
    });
    let search_pack = json!({
        "schemaVersion": "neonei/rust-search-pack/current",
        "sourceKind": "dist-data",
        "files": runtime_file_descriptors(
            input,
            &manifest,
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
    let browser_items = read_json_collection(
        input,
        &manifest,
        &["browserCatalog", "items"],
        Some("items"),
    )?;
    let browser_index_by_item = browser_items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| Some((value_string(item, "itemId")?, index as u64)))
        .collect::<BTreeMap<_, _>>();
    let mut search_rows = read_json_collection(
        input,
        &manifest,
        &["searchAll", "browserCatalog", "items"],
        Some("items"),
    )?;
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
