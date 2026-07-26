use crate::binary::{intern_compact_string, push_u32, write_binary_pack_payload};
use crate::io::write_json_value;
use crate::json_ext::{value_string, value_u64};
use crate::manifest::{
    COLLECTION_BROWSER_GROUPS, COLLECTION_BROWSER_ITEMS_PREFER_CATALOG, COLLECTION_SEARCH_ITEMS,
};
use crate::session::RawExportSession;
use crate::text::{build_pinyin_fields, normalize_search_terms, normalize_text};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct RawSearchItem {
    item_id: String,
    localized_name: Option<String>,
    mod_id: Option<String>,
    internal_name: Option<String>,
    render_asset_ref: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct BrowserGroupRow {
    group_key: Option<String>,
    group_label: Option<String>,
    representative_item_id: Option<String>,
    member_item_ids: Vec<String>,
    group_size: Option<u64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchItemRow {
    item_id: String,
    public_item_id: String,
    localized_name: Option<String>,
    mod_id: Option<String>,
    internal_name: Option<String>,
    normalized_localized_name: String,
    normalized_internal_name: String,
    normalized_item_id: String,
    normalized_search_terms: String,
    pinyin_full: String,
    pinyin_acronym: String,
    aliases: String,
    popularity_score: u64,
    search_rank: usize,
    render_asset_ref: Option<String>,
    group_key: Option<String>,
    group_label: Option<String>,
    group_size: u64,
    representative_item_id: String,
}
pub fn build_compact_string_payload_from_items(items: &[Value]) -> Result<Vec<u8>> {
    let mut strings = vec![String::new()];
    let mut string_refs = HashMap::new();
    string_refs.insert(String::new(), 0u32);
    let mut rows = Vec::<[u32; 6]>::with_capacity(items.len());

    let mut sorted_items = items.to_vec();
    sorted_items.sort_by_key(|left| value_string(left, "itemId"));

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
    session: &RawExportSession,
    output: &Path,
    strict: bool,
    debug_json: bool,
) -> Result<()> {
    let items = session.read_manifest_collection(COLLECTION_SEARCH_ITEMS)?;
    let group_rows = session.read_manifest_collection(COLLECTION_BROWSER_GROUPS)?;

    if strict && items.is_empty() {
        return Err(anyhow!(
            "search compiler blocked: raw items stream is empty"
        ));
    }

    let mut group_by_member = BTreeMap::<String, BrowserGroupRow>::new();
    for row in group_rows.iter() {
        let mut group: BrowserGroupRow = serde_json::from_value(row.clone())?;
        if group.group_size.is_none() {
            group.group_size = Some(group.member_item_ids.len() as u64);
        }
        for member in &group.member_item_ids {
            group_by_member.insert(member.clone(), group.clone());
        }
    }

    let mut alias_items = 0usize;
    let search_items = items
        .iter()
        .enumerate()
        .map(|(search_rank, item)| {
            let item: RawSearchItem = serde_json::from_value(item.clone())?;
            let item_id = item.item_id;
            let localized_name = item.localized_name;
            let mod_id = item.mod_id;
            let internal_name = item.internal_name;
            let render_asset_ref = item.render_asset_ref;
            let (pinyin_full, pinyin_acronym) =
                build_pinyin_fields(localized_name.as_deref().unwrap_or_default());
            let group = group_by_member.get(&item_id);
            let public_item_id = format!("item:{}", item_id.to_ascii_lowercase());
            let curated_aliases = normalize_search_terms(
                [
                    group.and_then(|value| value.group_key.as_deref()),
                    group.and_then(|value| value.group_label.as_deref()),
                ]
                .into_iter()
                .flatten(),
            );
            let normalized_terms = normalize_search_terms(
                [
                    localized_name.as_deref(),
                    internal_name.as_deref(),
                    mod_id.as_deref(),
                    Some(public_item_id.as_str()),
                    group.and_then(|value| value.group_key.as_deref()),
                    group.and_then(|value| value.group_label.as_deref()),
                ]
                .into_iter()
                .flatten(),
            );
            alias_items += 1;
            Ok(SearchItemRow {
                normalized_localized_name: localized_name
                    .as_deref()
                    .map(normalize_text)
                    .unwrap_or_default(),
                normalized_internal_name: internal_name
                    .as_deref()
                    .map(normalize_text)
                    .unwrap_or_default(),
                normalized_item_id: normalize_text(&item_id),
                aliases: curated_aliases,
                popularity_score: group.and_then(|value| value.group_size).unwrap_or(1),
                group_key: group.and_then(|value| value.group_key.clone()),
                group_label: group.and_then(|value| value.group_label.clone()),
                group_size: group.and_then(|value| value.group_size).unwrap_or(1),
                representative_item_id: group
                    .and_then(|value| value.representative_item_id.clone())
                    .unwrap_or_else(|| item_id.clone()),
                item_id,
                public_item_id,
                localized_name,
                mod_id,
                internal_name,
                normalized_search_terms: normalized_terms,
                pinyin_full,
                pinyin_acronym,
                search_rank,
                render_asset_ref,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let rust_dir = output.join("rust");
    fs::create_dir_all(&rust_dir)?;
    let browser_items =
        session.read_manifest_collection(COLLECTION_BROWSER_ITEMS_PREFER_CATALOG)?;
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
