use crate::binary::{intern_compact_string, push_u32, write_binary_pack_payload};
use crate::io::write_json_value;
use crate::json_ext::{value_string, value_u64};
use crate::manifest::{read_json_collection, read_manifest};
use crate::text::{build_pinyin_fields, normalize_search_terms, normalize_text};
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;
pub fn build_compact_string_payload_from_items(items: &[Value]) -> Result<Vec<u8>> {
    let mut strings = vec![String::new()];
    let mut string_refs = HashMap::new();
    string_refs.insert(String::new(), 0u32);
    let mut rows = Vec::<[u32; 6]>::with_capacity(items.len());

    let mut sorted_items = items.to_vec();
    sorted_items
        .sort_by(|left, right| value_string(left, "itemId").cmp(&value_string(right, "itemId")));

    for item in &sorted_items {
        rows.push([
            intern_compact_string(&mut strings, &mut string_refs, value_string(item, "itemId")),
            intern_compact_string(
                &mut strings,
                &mut string_refs,
                value_string(item, "localizedName"),
            ),
            intern_compact_string(&mut strings, &mut string_refs, value_string(item, "modId")),
            intern_compact_string(
                &mut strings,
                &mut string_refs,
                value_string(item, "internalName"),
            ),
            intern_compact_string(
                &mut strings,
                &mut string_refs,
                value_string(item, "groupKey"),
            ),
            intern_compact_string(
                &mut strings,
                &mut string_refs,
                value_string(item, "groupLabel"),
            ),
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
    payload.extend_from_slice(b"NEISTR1\0");
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

pub fn build_compact_search_payload_from_items(items: &[Value]) -> Result<Vec<u8>> {
    let mut strings = vec![String::new()];
    let mut string_refs = HashMap::new();
    string_refs.insert(String::new(), 0u32);
    let mut rows = Vec::<[u32; 13]>::with_capacity(items.len());

    let mut sorted_items = items.to_vec();
    sorted_items.sort_by(|left, right| {
        value_u64(left, "searchRank")
            .cmp(&value_u64(right, "searchRank"))
            .then_with(|| value_string(left, "itemId").cmp(&value_string(right, "itemId")))
    });
    for (fallback_rank, item) in sorted_items.iter().enumerate() {
        let popularity = value_u64(item, "popularityScore")
            .unwrap_or(1)
            .min(u32::MAX as u64) as u32;
        let search_rank = value_u64(item, "searchRank")
            .unwrap_or(fallback_rank as u64)
            .min(u32::MAX as u64) as u32;
        let browser_index = value_u64(item, "browserIndex")
            .unwrap_or(search_rank as u64)
            .min(u32::MAX as u64) as u32;
        rows.push([
            intern_compact_string(&mut strings, &mut string_refs, value_string(item, "itemId")),
            intern_compact_string(
                &mut strings,
                &mut string_refs,
                value_string(item, "publicItemId"),
            ),
            intern_compact_string(
                &mut strings,
                &mut string_refs,
                value_string(item, "localizedName"),
            ),
            intern_compact_string(&mut strings, &mut string_refs, value_string(item, "modId")),
            intern_compact_string(
                &mut strings,
                &mut string_refs,
                value_string(item, "normalizedLocalizedName"),
            ),
            intern_compact_string(
                &mut strings,
                &mut string_refs,
                value_string(item, "normalizedInternalName"),
            ),
            intern_compact_string(
                &mut strings,
                &mut string_refs,
                value_string(item, "normalizedItemId"),
            ),
            intern_compact_string(
                &mut strings,
                &mut string_refs,
                value_string(item, "normalizedSearchTerms"),
            ),
            intern_compact_string(
                &mut strings,
                &mut string_refs,
                value_string(item, "pinyinFull"),
            ),
            intern_compact_string(
                &mut strings,
                &mut string_refs,
                value_string(item, "pinyinAcronym"),
            ),
            popularity,
            search_rank,
            browser_index,
        ]);
    }

    let mut string_offsets = Vec::<u32>::with_capacity(strings.len());
    let mut string_bytes = Vec::<u8>::new();
    for value in &strings {
        string_offsets.push(string_bytes.len() as u32);
        string_bytes.extend_from_slice(value.as_bytes());
        string_bytes.push(0);
    }

    let row_stride_u32 = 13u32;
    let mut payload = Vec::with_capacity(
        8 + 4 * 4
            + string_offsets.len() * 4
            + rows.len() * row_stride_u32 as usize * 4
            + string_bytes.len(),
    );
    payload.extend_from_slice(b"NEISRC2\0");
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

pub fn compile_search_pack(
    input: &Path,
    output: &Path,
    strict: bool,
    debug_json: bool,
) -> Result<()> {
    let manifest = read_manifest(input)?;
    let items = read_json_collection(
        input,
        &manifest,
        &["items", "browserCatalog", "searchAll"],
        Some("items"),
    )?;
    let group_rows = read_json_collection(
        input,
        &manifest,
        &["groups", "browserGroups"],
        Some("groups"),
    )?;

    if strict && items.is_empty() {
        return Err(anyhow!(
            "search compiler blocked: raw items stream is empty"
        ));
    }

    let mut group_by_member = BTreeMap::new();
    for row in &group_rows {
        let group_key = value_string(row, "groupKey");
        let group_label = value_string(row, "groupLabel");
        let representative = value_string(row, "representativeItemId");
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
        for member in &members {
            group_by_member.insert(
                member.clone(),
                json!({
                    "groupKey": group_key,
                    "groupLabel": group_label,
                    "groupSize": value_u64(row, "groupSize").unwrap_or(members.len() as u64),
                    "representativeItemId": representative,
                }),
            );
        }
    }

    let mut alias_items = 0usize;
    let search_items = items
        .iter()
        .enumerate()
        .map(|(search_rank, item)| {
            let item_id = value_string(item, "itemId").unwrap_or_default();
            let localized_name = value_string(item, "localizedName");
            let mod_id = value_string(item, "modId");
            let internal_name = value_string(item, "internalName");
            let render_asset_ref = value_string(item, "renderAssetRef");
            let raw_search_terms = value_string(item, "searchTerms");
            let (pinyin_full, pinyin_acronym) =
                build_pinyin_fields(localized_name.as_deref().unwrap_or_default());
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
            alias_items += 1;
            json!({
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
            })
        })
        .collect::<Vec<_>>();

    let rust_dir = output.join("rust");
    fs::create_dir_all(&rust_dir)?;
    let browser_items = read_json_collection(
        input,
        &manifest,
        &["browserCatalog", "items"],
        Some("items"),
    )?;
    let string_pack = build_compact_string_payload_from_items(&browser_items)?;
    let search_pack = json!({
        "schemaVersion": "neonei/rust-search-pack/current",
        "counts": {
            "items": search_items.len(),
            "aliasItems": alias_items,
        },
        "items": search_items,
    });
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
    write_binary_pack_payload(
        &rust_dir.join("strings.zh_cn.bin"),
        "neonei/string-pack/current",
        &string_pack,
    )?;
    Ok(())
}
