use crate::atlas_repair::repaired_browser_atlas;
use crate::binary::{
    intern_compact_string, push_u32, write_binary_pack, write_binary_pack_payload,
};
use crate::io::write_json_value;
use crate::json_ext::{
    numeric_value_u64, optional_value_string, optional_value_u64, value_string, value_u64,
};
use crate::manifest::{
    read_json_collection, read_manifest, read_manifest_json, runtime_file_descriptors,
};
use crate::texture_animation::{
    expected_animated_item, expected_animation_reason, promote_animation_facts_to_animated_atlas,
};
use crate::validation::{validate_atlas_bounds, validate_atlas_ref, validate_frame_bounds};
use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Default)]
struct AtlasMetaRow {
    atlas_file: String,
    width: u64,
    height: u64,
    kind_flags: u32,
    item_count: u32,
    frame_count: u32,
}

pub fn build_compact_atlas_meta_payload_from_atlas_items(atlas_items: &[Value]) -> Result<Vec<u8>> {
    let mut atlas_rows = BTreeMap::<String, AtlasMetaRow>::new();
    for item in atlas_items {
        if let Some(static_atlas) = item.get("staticAtlas").filter(|value| value.is_object()) {
            note_atlas_meta(&mut atlas_rows, static_atlas, 1, 0);
        }
        if let Some(animated_atlas) = item.get("animatedAtlas").filter(|value| value.is_object()) {
            let frame_count = animated_atlas
                .get("frames")
                .and_then(Value::as_array)
                .map(|values| values.len() as u32)
                .or_else(|| value_u64(animated_atlas, "frameCount").map(|value| value as u32))
                .unwrap_or(0);
            note_atlas_meta(&mut atlas_rows, animated_atlas, 2, frame_count);
        }
    }

    let mut strings = vec![String::new()];
    let mut string_refs = HashMap::new();
    string_refs.insert(String::new(), 0u32);
    let mut rows = Vec::<[u32; 6]>::new();
    for row in atlas_rows.values() {
        let atlas_file =
            intern_compact_string(&mut strings, &mut string_refs, Some(row.atlas_file.clone()));
        rows.push([
            atlas_file,
            row.width.min(u32::MAX as u64) as u32,
            row.height.min(u32::MAX as u64) as u32,
            row.kind_flags,
            row.item_count,
            row.frame_count,
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
    payload.extend_from_slice(b"NEIATM1\0");
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

pub fn normalize_runtime_atlas_file_path(value: Option<String>) -> Option<String> {
    let normalized = value?
        .replace('\\', "/")
        .trim_start_matches('/')
        .to_string();
    if normalized.is_empty() {
        return None;
    }
    if let Some(stripped) = normalized.strip_prefix("assets/textures/") {
        return Some(format!("textures/{stripped}"));
    }
    Some(normalized)
}

pub fn copy_runtime_atlas_assets(
    input: &Path,
    output: &Path,
    atlas_items: &[Value],
    missing_atlas_asset_files: &mut Vec<String>,
) -> Result<()> {
    let mut atlas_paths = BTreeMap::<String, String>::new();
    for item in atlas_items {
        for key in ["staticAtlas", "animatedAtlas"] {
            let Some(atlas) = item.get(key).filter(|value| value.is_object()) else {
                continue;
            };
            let Some(raw_atlas_file) = optional_value_string(Some(atlas), "atlasFile") else {
                continue;
            };
            let Some(runtime_atlas_file) =
                normalize_runtime_atlas_file_path(Some(raw_atlas_file.clone()))
            else {
                continue;
            };
            atlas_paths
                .entry(runtime_atlas_file)
                .or_insert(raw_atlas_file);
        }
    }

    for (runtime_atlas_file, raw_atlas_file) in atlas_paths {
        let raw_relative = raw_atlas_file
            .replace('\\', "/")
            .trim_start_matches('/')
            .to_string();
        let source_path = input.join(&raw_relative);
        if !source_path.is_file() {
            missing_atlas_asset_files.push(format!(
                "{runtime_atlas_file}:missing-source:{raw_relative}"
            ));
            continue;
        }
        let runtime_relative = runtime_atlas_file
            .replace('\\', "/")
            .trim_start_matches('/')
            .to_string();
        let destination_path = output.join(&runtime_relative);
        if let Some(parent) = destination_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&source_path, &destination_path).with_context(|| {
            format!(
                "copy runtime atlas asset {} -> {}",
                source_path.display(),
                destination_path.display()
            )
        })?;
    }
    Ok(())
}

pub fn compile_texture_pack(
    input: &Path,
    output: &Path,
    strict: bool,
    debug_json: bool,
) -> Result<()> {
    let manifest = read_manifest(input)?;
    if manifest.files.contains_key("textureManifest") {
        return compile_dist_texture_pack(input, output, strict, debug_json);
    }
    let atlas = read_manifest_json(input, &manifest, "browserAtlasIndex")?
        .ok_or_else(|| anyhow!("texture compiler blocked: browserAtlasIndex is missing"))?;
    let animations = read_json_collection(
        input,
        &manifest,
        &["animations", "animationTable"],
        Some("animations"),
    )?;
    let native_sprites = read_json_collection(
        input,
        &manifest,
        &["nativeSprites", "nativeRenderIndex"],
        Some("sprites"),
    )?;
    let texture_rows = read_json_collection(
        input,
        &manifest,
        &["textures", "textureManifest"],
        Some("textures"),
    )?;
    let item_rows = read_json_collection(
        input,
        &manifest,
        &["items", "browserCatalog"],
        Some("items"),
    )?;

    let animation_by_asset = animations
        .iter()
        .filter_map(|row| Some((value_string(row, "assetId")?, row.clone())))
        .collect::<BTreeMap<_, _>>();
    let native_sprite_by_asset = native_sprites
        .iter()
        .filter_map(|row| Some((value_string(row, "assetId")?, row.clone())))
        .collect::<BTreeMap<_, _>>();
    let texture_by_asset = texture_rows
        .iter()
        .filter_map(|row| Some((value_string(row, "assetId")?, row.clone())))
        .collect::<BTreeMap<_, _>>();

    let atlas = repaired_browser_atlas(&atlas, &texture_rows, &item_rows);
    let atlas = promote_animation_facts_to_animated_atlas(
        &atlas,
        &animation_by_asset,
        &native_sprite_by_asset,
    );
    let atlas_items = atlas
        .get("items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut static_items = 0u64;
    let mut animated_items = 0u64;
    let mut missing_atlas_file_refs = Vec::new();
    let mut missing_atlas_asset_files = Vec::new();
    let mut invalid_atlas_bounds = Vec::new();
    let mut invalid_frame_bounds = Vec::new();
    let mut actionable_texture_issues = Vec::new();
    let mut animation_table = Vec::new();
    let mut atlas_map = BTreeMap::new();

    for item in &atlas_items {
        let item_id = value_string(item, "itemId").unwrap_or_default();
        let asset_id = value_string(item, "assetId").unwrap_or_default();
        let has_static_atlas = item
            .get("hasStaticAtlas")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let has_animated_atlas = item
            .get("hasAnimatedAtlas")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let animation = animation_by_asset.get(&asset_id);
        let native_sprite = native_sprite_by_asset.get(&asset_id);
        if !has_static_atlas && !has_animated_atlas {
            actionable_texture_issues.push(json!({
                "code": "TEXTURE_ATLAS_ENTRY_EMPTY",
                "itemId": item_id,
                "assetId": asset_id,
                "reason": "No staticAtlas or animatedAtlas was generated for this browser atlas item.",
                "recommendedFix": "Fix NESQL++ texture capture or atlas-source classification for this item; do not use frontend per-item image fallback.",
            }));
        }
        if !has_animated_atlas && expected_animated_item(animation, native_sprite) {
            actionable_texture_issues.push(json!({
                "code": "EXPECTED_ANIMATED_BUT_STATIC",
                "itemId": item_id,
                "assetId": asset_id,
                "reason": expected_animation_reason(animation, native_sprite),
                "hasStaticAtlas": has_static_atlas,
                "hasAnimationFacts": animation.is_some(),
                "hasNativeSpriteFacts": native_sprite.is_some(),
                "recommendedFix": "Repair NESQL++ animation facts or native sprite capture so compiler emits animatedAtlas/timeline rows.",
            }));
        }
        if item
            .get("hasStaticAtlas")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            static_items += 1;
            validate_atlas_ref(
                &item_id,
                item.get("staticAtlas"),
                &mut missing_atlas_file_refs,
            );
            validate_atlas_bounds(
                &item_id,
                "static",
                item.get("staticAtlas"),
                &mut invalid_atlas_bounds,
            );
        }
        if item
            .get("hasAnimatedAtlas")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            animated_items += 1;
            let animated_atlas = item.get("animatedAtlas");
            validate_atlas_ref(&item_id, animated_atlas, &mut missing_atlas_file_refs);
            validate_atlas_bounds(
                &item_id,
                "animated",
                animated_atlas,
                &mut invalid_atlas_bounds,
            );
            validate_frame_bounds(&item_id, animated_atlas, &mut invalid_frame_bounds);

            let frame_duration_ms = animated_atlas
                .and_then(|value| value_u64(value, "frameDurationMs"))
                .or_else(|| animation.and_then(|value| value_u64(value, "frameDurationMs")))
                .or_else(|| native_sprite.and_then(|value| value_u64(value, "frameDurationMs")));
            animation_table.push(json!({
                "itemId": item_id,
                "assetId": asset_id,
                "mode": native_sprite.and_then(|value| value.get("animationMode")).cloned().unwrap_or(Value::Null),
                "frameDurationSource": if native_sprite.is_some() { "native_sprite_metadata" } else { "raw_animation_index" },
                "frameCount": animated_atlas
                    .and_then(|value| value_u64(value, "frameCount"))
                    .or_else(|| animation.and_then(|value| value_u64(value, "frameCount"))),
                "frameDurationMs": frame_duration_ms,
                "atlasFile": animated_atlas
                    .and_then(|value| value.get("atlasFile"))
                    .cloned()
                    .unwrap_or(Value::Null),
                "timeline": normalize_timeline(animated_atlas, frame_duration_ms),
                "spriteMetadataFile": native_sprite
                    .and_then(|value| value.get("spriteMetadataFile"))
                    .cloned()
                    .unwrap_or(Value::Null),
            }));
        }
        atlas_map.insert(
            item_id,
            json!({
                "assetId": asset_id,
                "atlas": item,
                "texture": texture_by_asset.get(&asset_id).cloned().unwrap_or(Value::Null),
            }),
        );
    }
    copy_runtime_atlas_assets(input, output, &atlas_items, &mut missing_atlas_asset_files)?;

    if strict
        && (!missing_atlas_file_refs.is_empty()
            || !missing_atlas_asset_files.is_empty()
            || !invalid_atlas_bounds.is_empty()
            || !invalid_frame_bounds.is_empty())
    {
        return Err(anyhow!(
            "texture compiler blocked: missing atlas refs={}, missing atlas assets={}, invalid atlas bounds={}, invalid frame bounds={}",
            missing_atlas_file_refs.len(),
            missing_atlas_asset_files.len(),
            invalid_atlas_bounds.len(),
            invalid_frame_bounds.len()
        ));
    }

    let rust_dir = output.join("rust");
    fs::create_dir_all(&rust_dir)?;
    let texture_output_pack = json!({
        "schemaVersion": "neonei/rust-texture-pack/current",
        "counts": {
            "atlasItems": atlas_items.len(),
            "staticAtlasItems": static_items,
            "animatedAtlasItems": animated_items,
            "animationRows": animations.len(),
            "nativeSpriteRows": native_sprites.len(),
            "textureRows": texture_rows.len(),
            "missingAtlasFileRefs": missing_atlas_file_refs.len(),
            "missingAtlasAssetFiles": missing_atlas_asset_files.len(),
            "invalidAtlasBounds": invalid_atlas_bounds.len(),
            "invalidFrameBounds": invalid_frame_bounds.len(),
            "atlasMapItems": atlas_map.len(),
        },
        "atlas": atlas,
        "atlasMap": atlas_map,
        "animationTable": animation_table,
        "validation": {
            "missingAtlasFileRefs": missing_atlas_file_refs.clone(),
            "missingAtlasAssetFiles": missing_atlas_asset_files.clone(),
            "invalidAtlasBounds": invalid_atlas_bounds.clone(),
            "invalidFrameBounds": invalid_frame_bounds.clone(),
        },
    });
    let texture_report_status = if actionable_texture_issues.is_empty()
        && missing_atlas_file_refs.is_empty()
        && missing_atlas_asset_files.is_empty()
        && invalid_atlas_bounds.is_empty()
        && invalid_frame_bounds.is_empty()
    {
        "ok"
    } else {
        "advisory"
    };
    let actionable_issue_count = actionable_texture_issues.len() + missing_atlas_asset_files.len();
    let suspicious_texture_report = json!({
        "schemaVersion": "neonei/rust-suspicious-texture-report/current",
        "generatedAt": "deterministic-rust-compiler",
        "status": texture_report_status,
        "counts": {
            "actionableIssues": actionable_issue_count,
            "missingAtlasFileRefs": missing_atlas_file_refs.len(),
            "missingAtlasAssetFiles": missing_atlas_asset_files.len(),
            "invalidAtlasBounds": invalid_atlas_bounds.len(),
            "invalidFrameBounds": invalid_frame_bounds.len(),
        },
        "issues": actionable_texture_issues,
        "missingAtlasFileRefs": missing_atlas_file_refs,
        "missingAtlasAssetFiles": missing_atlas_asset_files,
        "invalidAtlasBounds": invalid_atlas_bounds,
        "invalidFrameBounds": invalid_frame_bounds,
    });
    write_json_value(
        &rust_dir.join("missing-texture-report.json"),
        &json!({
            "schemaVersion": "neonei/rust-missing-texture-report/current",
            "generatedAt": "deterministic-rust-compiler",
            "status": texture_report_status,
            "counts": {
                "atlasItems": atlas_items.len(),
                "staticAtlasItems": static_items,
                "animatedAtlasItems": animated_items,
                "actionableIssues": suspicious_texture_report["counts"]["actionableIssues"].clone(),
                "missingAtlasFileRefs": suspicious_texture_report["counts"]["missingAtlasFileRefs"].clone(),
                "missingAtlasAssetFiles": suspicious_texture_report["counts"]["missingAtlasAssetFiles"].clone(),
                "invalidAtlasBounds": suspicious_texture_report["counts"]["invalidAtlasBounds"].clone(),
                "invalidFrameBounds": suspicious_texture_report["counts"]["invalidFrameBounds"].clone(),
            },
            "issues": suspicious_texture_report["issues"].clone(),
            "missingAtlasFileRefs": suspicious_texture_report["missingAtlasFileRefs"].clone(),
            "missingAtlasAssetFiles": suspicious_texture_report["missingAtlasAssetFiles"].clone(),
            "invalidAtlasBounds": suspicious_texture_report["invalidAtlasBounds"].clone(),
            "invalidFrameBounds": suspicious_texture_report["invalidFrameBounds"].clone(),
        }),
    )?;
    write_json_value(
        &rust_dir.join("suspicious-texture-report.json"),
        &suspicious_texture_report,
    )?;
    if debug_json {
        write_json_value(&rust_dir.join("texture-pack.json"), &texture_output_pack)?;
    }
    let texture_payload = build_compact_texture_payload_from_atlas_items(&atlas_items)?;
    write_binary_pack_payload(
        &rust_dir.join("textures.bin"),
        "neonei/texture-pack/current",
        &texture_payload,
    )?;
    let animation_payload = build_compact_animation_payload_from_table(&animation_table)?;
    write_binary_pack_payload(
        &rust_dir.join("animations.bin"),
        "neonei/animation-pack/current",
        &animation_payload,
    )?;
    let atlas_meta_payload = build_compact_atlas_meta_payload_from_atlas_items(&atlas_items)?;
    write_binary_pack_payload(
        &rust_dir.join("atlas.meta.bin"),
        "neonei/atlas-meta-pack/current",
        &atlas_meta_payload,
    )?;
    Ok(())
}

pub fn compile_dist_texture_pack(
    input: &Path,
    output: &Path,
    strict: bool,
    debug_json: bool,
) -> Result<()> {
    let manifest = read_manifest(input)?;
    let texture_files = runtime_file_descriptors(
        input,
        &manifest,
        &[
            ("textureManifest", "textureManifest"),
            ("browserAtlasIndex", "browserAtlasIndex"),
            ("nativeRenderIndex", "nativeRenderIndex"),
            ("animationTable", "animationTable"),
            ("animationExpectationReport", "animationExpectationReport"),
        ],
    )?;
    if strict
        && !texture_files.iter().any(|value| {
            value
                .get("logicalName")
                .and_then(Value::as_str)
                .is_some_and(|value| value == "textureManifest")
        })
    {
        return Err(anyhow!(
            "texture compiler blocked: textureManifest is missing"
        ));
    }
    let texture_pack = json!({
        "schemaVersion": "neonei/rust-texture-pack/current",
        "sourceKind": "dist-data",
        "counts": { "files": texture_files.len() },
        "files": texture_files,
    });
    let animation_pack = json!({
        "schemaVersion": "neonei/rust-animation-pack/current",
        "sourceKind": "dist-data",
        "files": runtime_file_descriptors(
            input,
            &manifest,
            &[("animationTable", "animationTable"), ("animationExpectationReport", "animationExpectationReport")],
        )?,
    });

    let rust_dir = output.join("rust");
    fs::create_dir_all(&rust_dir)?;
    if debug_json {
        write_json_value(&rust_dir.join("texture-pack.json"), &texture_pack)?;
    }
    write_binary_pack(
        &rust_dir.join("textures.bin"),
        "neonei/texture-pack/current",
        &texture_pack,
    )?;
    write_binary_pack(
        &rust_dir.join("animations.bin"),
        "neonei/animation-pack/current",
        &animation_pack,
    )?;
    let atlas_meta_payload = build_compact_atlas_meta_payload_from_atlas_items(&[])?;
    write_binary_pack_payload(
        &rust_dir.join("atlas.meta.bin"),
        "neonei/atlas-meta-pack/current",
        &atlas_meta_payload,
    )?;
    Ok(())
}

fn note_atlas_meta(
    atlas_rows: &mut BTreeMap<String, AtlasMetaRow>,
    atlas: &Value,
    kind_flag: u32,
    frame_count: u32,
) {
    let Some(atlas_file) =
        normalize_runtime_atlas_file_path(optional_value_string(Some(atlas), "atlasFile"))
    else {
        return;
    };
    let width = value_u64(atlas, "atlasWidth")
        .or_else(|| {
            let x = value_u64(atlas, "x")?;
            let width = value_u64(atlas, "width")?;
            Some(x.saturating_add(width))
        })
        .unwrap_or(0);
    let height = value_u64(atlas, "atlasHeight")
        .or_else(|| {
            let y = value_u64(atlas, "y")?;
            let height = value_u64(atlas, "height")?;
            Some(y.saturating_add(height))
        })
        .unwrap_or(0);
    let row = atlas_rows
        .entry(atlas_file.clone())
        .or_insert_with(|| AtlasMetaRow {
            atlas_file,
            ..AtlasMetaRow::default()
        });
    row.width = row.width.max(width);
    row.height = row.height.max(height);
    row.kind_flags |= kind_flag;
    row.item_count = row.item_count.saturating_add(1);
    row.frame_count = row.frame_count.saturating_add(frame_count);
}

pub fn build_compact_animation_payload_from_table(animation_table: &[Value]) -> Result<Vec<u8>> {
    let mut strings = vec![String::new()];
    let mut string_refs = HashMap::new();
    string_refs.insert(String::new(), 0u32);
    let mut rows = Vec::<[u32; 5]>::new();
    let mut frames = Vec::<[u32; 2]>::new();

    let mut sorted_animations = animation_table.to_vec();
    sorted_animations
        .sort_by(|left, right| value_string(left, "itemId").cmp(&value_string(right, "itemId")));

    for animation in &sorted_animations {
        let item_id = intern_compact_string(
            &mut strings,
            &mut string_refs,
            value_string(animation, "itemId"),
        );
        let atlas_file = intern_compact_string(
            &mut strings,
            &mut string_refs,
            normalize_runtime_atlas_file_path(value_string(animation, "atlasFile")),
        );
        let frame_start = frames.len() as u32;
        let frame_duration_ms = value_u64(animation, "frameDurationMs").unwrap_or(0) as u32;
        let timeline = animation
            .get("timeline")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        for (index, frame) in timeline.iter().enumerate() {
            frames.push([
                value_u64(frame, "frameIndex").unwrap_or(index as u64) as u32,
                value_u64(frame, "durationMs")
                    .unwrap_or(frame_duration_ms as u64)
                    .max(16) as u32,
            ]);
        }
        rows.push([
            item_id,
            atlas_file,
            frame_start,
            (frames.len() as u32).saturating_sub(frame_start),
            frame_duration_ms,
        ]);
    }

    let mut string_offsets = Vec::<u32>::with_capacity(strings.len());
    let mut string_bytes = Vec::<u8>::new();
    for value in &strings {
        string_offsets.push(string_bytes.len() as u32);
        string_bytes.extend_from_slice(value.as_bytes());
        string_bytes.push(0);
    }

    let row_stride_u32 = 5u32;
    let frame_stride_u32 = 2u32;
    let mut payload = Vec::with_capacity(
        8 + 6 * 4
            + string_offsets.len() * 4
            + rows.len() * row_stride_u32 as usize * 4
            + frames.len() * frame_stride_u32 as usize * 4
            + string_bytes.len(),
    );
    payload.extend_from_slice(b"NEIANM1\0");
    push_u32(&mut payload, 1);
    push_u32(&mut payload, rows.len() as u32);
    push_u32(&mut payload, strings.len() as u32);
    push_u32(&mut payload, frames.len() as u32);
    push_u32(&mut payload, row_stride_u32);
    push_u32(&mut payload, frame_stride_u32);
    for offset in string_offsets {
        push_u32(&mut payload, offset);
    }
    for row in rows {
        for value in row {
            push_u32(&mut payload, value);
        }
    }
    for frame in frames {
        for value in frame {
            push_u32(&mut payload, value);
        }
    }
    payload.extend_from_slice(&string_bytes);
    Ok(payload)
}

pub fn build_compact_texture_payload_from_atlas_items(atlas_items: &[Value]) -> Result<Vec<u8>> {
    let mut strings = vec![String::new()];
    let mut string_refs = HashMap::new();
    string_refs.insert(String::new(), 0u32);
    let mut rows = Vec::<[u32; 10]>::new();
    let mut frames = Vec::<[u32; 5]>::new();

    let mut sorted_items = atlas_items.to_vec();
    sorted_items
        .sort_by(|left, right| value_string(left, "itemId").cmp(&value_string(right, "itemId")));

    for item in &sorted_items {
        let item_id =
            intern_compact_string(&mut strings, &mut string_refs, value_string(item, "itemId"));
        let static_atlas = item.get("staticAtlas").filter(|value| value.is_object());
        let animated_atlas = item.get("animatedAtlas").filter(|value| value.is_object());
        let static_file = intern_compact_string(
            &mut strings,
            &mut string_refs,
            normalize_runtime_atlas_file_path(optional_value_string(static_atlas, "atlasFile")),
        );
        let animated_file = intern_compact_string(
            &mut strings,
            &mut string_refs,
            normalize_runtime_atlas_file_path(optional_value_string(animated_atlas, "atlasFile")),
        );
        let frame_start = frames.len() as u32;
        let frame_duration_ms =
            optional_value_u64(animated_atlas, "frameDurationMs").unwrap_or(0) as u32;

        if let Some(animated_atlas) = animated_atlas {
            let frame_values = animated_atlas
                .get("frames")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let timeline = normalize_timeline(
                Some(animated_atlas),
                optional_value_u64(Some(animated_atlas), "frameDurationMs"),
            );
            let timeline_values = timeline.as_array().cloned().unwrap_or_default();
            for (index, frame) in frame_values.iter().enumerate() {
                let duration_ms = timeline_values
                    .get(index)
                    .and_then(|value| value_u64(value, "durationMs"))
                    .unwrap_or(frame_duration_ms as u64)
                    .max(16) as u32;
                frames.push([
                    value_u64(frame, "x").unwrap_or(0) as u32,
                    value_u64(frame, "y").unwrap_or(0) as u32,
                    value_u64(frame, "width").unwrap_or(0) as u32,
                    value_u64(frame, "height").unwrap_or(0) as u32,
                    duration_ms,
                ]);
            }
        }

        rows.push([
            item_id,
            static_file,
            optional_value_u64(static_atlas, "x").unwrap_or(0) as u32,
            optional_value_u64(static_atlas, "y").unwrap_or(0) as u32,
            optional_value_u64(static_atlas, "width").unwrap_or(0) as u32,
            optional_value_u64(static_atlas, "height").unwrap_or(0) as u32,
            animated_file,
            frame_start,
            (frames.len() as u32).saturating_sub(frame_start),
            frame_duration_ms,
        ]);
    }

    let mut string_offsets = Vec::<u32>::with_capacity(strings.len());
    let mut string_bytes = Vec::<u8>::new();
    for value in &strings {
        string_offsets.push(string_bytes.len() as u32);
        string_bytes.extend_from_slice(value.as_bytes());
        string_bytes.push(0);
    }

    let row_stride_u32 = 10u32;
    let frame_stride_u32 = 5u32;
    let mut payload = Vec::with_capacity(
        8 + 6 * 4
            + string_offsets.len() * 4
            + rows.len() * row_stride_u32 as usize * 4
            + frames.len() * frame_stride_u32 as usize * 4
            + string_bytes.len(),
    );
    payload.extend_from_slice(b"NEITEX1\0");
    push_u32(&mut payload, 1);
    push_u32(&mut payload, rows.len() as u32);
    push_u32(&mut payload, strings.len() as u32);
    push_u32(&mut payload, frames.len() as u32);
    push_u32(&mut payload, row_stride_u32);
    push_u32(&mut payload, frame_stride_u32);
    for offset in string_offsets {
        push_u32(&mut payload, offset);
    }
    for row in rows {
        for value in row {
            push_u32(&mut payload, value);
        }
    }
    for frame in frames {
        for value in frame {
            push_u32(&mut payload, value);
        }
    }
    payload.extend_from_slice(&string_bytes);
    Ok(payload)
}

pub fn normalize_timeline_frame_index(frame_index: u64, frame_count: u64) -> u64 {
    if frame_count > 0 && frame_index >= frame_count {
        return frame_index % frame_count;
    }
    frame_index
}

pub fn normalize_timeline(
    animated_atlas: Option<&Value>,
    fallback_duration_ms: Option<u64>,
) -> Value {
    let Some(animated_atlas) = animated_atlas else {
        return Value::Array(Vec::new());
    };
    let normalized_frame_count = animated_atlas
        .get("frames")
        .and_then(Value::as_array)
        .map(|frames| frames.len() as u64)
        .filter(|count| *count > 0)
        .or_else(|| animated_atlas.get("frameCount").and_then(numeric_value_u64))
        .unwrap_or(0);
    if let Some(timeline) = animated_atlas.get("timeline").and_then(Value::as_array) {
        return Value::Array(
            timeline
                .iter()
                .enumerate()
                .map(|(index, value)| {
                    if let Some(pair) = value.as_array() {
                        let frame_index = pair
                            .first()
                            .and_then(numeric_value_u64)
                            .unwrap_or(index as u64);
                        json!({
                            "frameIndex": normalize_timeline_frame_index(frame_index, normalized_frame_count),
                            "durationMs": pair.get(1).and_then(numeric_value_u64).or(fallback_duration_ms),
                        })
                    } else {
                        let frame_index = value
                            .get("frameIndex")
                            .or_else(|| value.get("index"))
                            .and_then(numeric_value_u64)
                            .unwrap_or(index as u64);
                        let duration_ms = value
                            .get("durationMs")
                            .and_then(numeric_value_u64)
                            .or(fallback_duration_ms);
                        json!({
                            "frameIndex": normalize_timeline_frame_index(frame_index, normalized_frame_count),
                            "durationMs": duration_ms,
                        })
                    }
                })
                .collect(),
        );
    }
    let frame_count = normalized_frame_count;
    Value::Array(
        (0..frame_count)
            .map(|frame_index| {
                json!({
                    "frameIndex": frame_index,
                    "durationMs": fallback_duration_ms,
                })
            })
            .collect(),
    )
}
