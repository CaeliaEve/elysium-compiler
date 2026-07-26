use crate::atlas_repair::repaired_browser_atlas_items;
use crate::binary::{
    intern_compact_string, push_u32, write_binary_pack, write_binary_pack_payload,
};
use crate::io::write_json_value;
use crate::json_ext::{
    numeric_value_u64, optional_value_string, optional_value_u64, value_string, value_u64,
};
use crate::manifest::{
    COLLECTION_ANIMATIONS, COLLECTION_ANIMATION_FRAME_MATERIALIZATIONS, COLLECTION_BROWSER_ITEMS,
    COLLECTION_FACADE_RESOLUTIONS, COLLECTION_NATIVE_SPRITES,
    COLLECTION_TEXTURE_ROWS_WITH_MANIFEST,
};
use crate::session::RawExportSession;
use crate::texture_animation::{
    animation_materialization_diagnostic, expected_animated_item, expected_animation_reason,
    promote_animation_facts_to_animated_atlas_items,
};
use crate::validation::{validate_atlas_bounds, validate_atlas_ref, validate_frame_bounds};
use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

const NATIVE_RENDER_INDEX_PATH: &str = "render/index.json";

#[derive(Clone, Debug, Default)]
struct NativeRenderIndexStats {
    texture_sprites: usize,
    item_renderers: usize,
    shader_items: usize,
    framebuffer_captures: usize,
    item_renderer_by_item_id: usize,
    shader_by_item_id: usize,
    sprite_by_icon_name: usize,
    shader_items_needing_capture: usize,
    missing_required_captures: usize,
    required_captures_without_frames: usize,
    required_captures_without_timeline: usize,
}

#[derive(Clone, Debug)]
struct ShaderCaptureRequirement {
    item_id: String,
    renderer_kind: Option<String>,
    renderer_class: Option<String>,
    shader_family: Option<String>,
    preferred_export: Option<String>,
}

type FramebufferCaptureIndex = (
    BTreeMap<String, Value>,
    BTreeMap<String, Value>,
    BTreeMap<String, CaptureProbe>,
    BTreeMap<String, CaptureProbe>,
);

fn index_framebuffer_captures(rows: &[Value]) -> Result<FramebufferCaptureIndex> {
    let mut values_by_asset = BTreeMap::<String, Value>::new();
    let mut values_by_variant = BTreeMap::<String, Value>::new();
    let mut probes_by_asset = BTreeMap::<String, CaptureProbe>::new();
    let mut probes_by_variant = BTreeMap::<String, CaptureProbe>::new();
    for row in rows {
        let compact = compact_framebuffer_capture(row);
        let probe = capture_probe_from_compact(&compact);
        if let Some(asset_id) = value_string(&compact, "assetId") {
            if values_by_asset.contains_key(&asset_id) {
                return Err(anyhow!(
                    "duplicate framebuffer capture assetId is forbidden: {}",
                    asset_id
                ));
            }
            values_by_asset.insert(asset_id.clone(), compact.clone());
            probes_by_asset.insert(asset_id, probe.clone());
        }
        if let Some(variant_key) = value_string(&compact, "variantKey") {
            if let Some(existing) = values_by_variant.get(&variant_key) {
                if equivalent_variant_capture(existing, &compact) {
                    continue;
                }
                return Err(anyhow!(
                    "conflicting framebuffer capture variantKey is forbidden: {}",
                    variant_key
                ));
            }
            values_by_variant.insert(variant_key.clone(), compact);
            probes_by_variant.insert(variant_key, probe);
        }
    }
    Ok((
        values_by_asset,
        values_by_variant,
        probes_by_asset,
        probes_by_variant,
    ))
}

fn equivalent_variant_capture(left: &Value, right: &Value) -> bool {
    let mut left = left.clone();
    let mut right = right.clone();
    if let Some(object) = left.as_object_mut() {
        object.remove("assetId");
    }
    if let Some(object) = right.as_object_mut() {
        object.remove("assetId");
    }
    left == right
}

#[derive(Clone, Debug)]
struct CaptureProbe {
    frame_count: u64,
    frames: usize,
    timeline: usize,
}

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

#[cfg(test)]
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

fn copy_runtime_atlas_assets_from_session(
    session: &RawExportSession,
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
        let Some(bytes) = session.read_relative_bytes(&raw_relative)? else {
            missing_atlas_asset_files.push(format!(
                "{runtime_atlas_file}:missing-source:{raw_relative}"
            ));
            continue;
        };
        let runtime_relative = runtime_atlas_file
            .replace('\\', "/")
            .trim_start_matches('/')
            .to_string();
        let destination_path = output.join(&runtime_relative);
        if let Some(parent) = destination_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&destination_path, bytes.as_slice())
            .with_context(|| format!("write runtime atlas asset {}", destination_path.display()))?;
    }
    Ok(())
}

fn stable_value_i64(value: Option<&Value>, fallback: i64) -> i64 {
    value
        .and_then(|value| {
            value
                .as_i64()
                .or_else(|| value.as_u64().and_then(|value| i64::try_from(value).ok()))
                .or_else(|| value.as_f64().map(|value| value as i64))
                .or_else(|| {
                    value
                        .as_str()
                        .and_then(|value| value.parse::<f64>().ok())
                        .map(|value| value as i64)
                })
        })
        .unwrap_or(fallback)
}

fn stable_value_u64(value: Option<&Value>, fallback: u64) -> u64 {
    let parsed = stable_value_i64(value, fallback as i64);
    if parsed < 0 {
        fallback
    } else {
        parsed as u64
    }
}

fn value_bool(value: &Value, key: &str) -> bool {
    value.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn value_or_null(value: &Value, key: &str) -> Value {
    value.get(key).cloned().unwrap_or(Value::Null)
}

fn compact_framebuffer_capture(row: &Value) -> Value {
    json!({
        "assetId": value_or_null(row, "assetId"),
        "variantKey": value_or_null(row, "variantKey"),
        "rendererFamily": value_or_null(row, "rendererFamily"),
        "renderMode": value_or_null(row, "renderMode"),
        "animationMode": value_or_null(row, "animationMode"),
        "captureMethod": value_or_null(row, "captureMethod"),
        "primaryArtifact": value_or_null(row, "primaryArtifact"),
        "framePattern": value_or_null(row, "framePattern"),
        "frameCount": stable_value_u64(
            row.get("frameCount"),
            stable_value_u64(row.get("capturedFrameCount"), 0),
        ),
        "frameDurationMs": stable_value_u64(row.get("frameDurationMs"), 50),
        "timeline": row.get("timeline").and_then(Value::as_array).cloned().unwrap_or_default(),
        "frames": row.get("frames").and_then(Value::as_array).cloned().unwrap_or_default(),
    })
}

fn normalize_animation_timeline(
    source_timeline: Option<&Value>,
    frame_count: u64,
    fallback_duration_ms: u64,
) -> Vec<Value> {
    let normalize_frame_index = |value: Option<&Value>, index: usize| -> u64 {
        let raw_index = stable_value_i64(value, index as i64);
        if raw_index < 0 {
            return index as u64;
        }
        let raw_index = raw_index as u64;
        if frame_count > 0 && raw_index >= frame_count {
            raw_index % frame_count
        } else {
            raw_index
        }
    };
    if let Some(timeline) = source_timeline
        .and_then(Value::as_array)
        .filter(|value| !value.is_empty())
    {
        return timeline
            .iter()
            .enumerate()
            .map(|(index, frame)| {
                let (frame_index, duration_ms) = if let Some(values) = frame.as_array() {
                    (
                        normalize_frame_index(values.first(), index),
                        stable_value_u64(values.get(1), fallback_duration_ms),
                    )
                } else {
                    (
                        normalize_frame_index(
                            frame.get("frameIndex").or_else(|| frame.get("index")),
                            index,
                        ),
                        stable_value_u64(
                            frame
                                .get("durationMs")
                                .or_else(|| frame.get("duration"))
                                .or_else(|| frame.get("timeMs")),
                            fallback_duration_ms,
                        ),
                    )
                };
                json!({
                    "frameIndex": frame_index,
                    "durationMs": duration_ms.max(16),
                })
            })
            .collect();
    }
    (0..frame_count)
        .map(|index| {
            json!({
                "frameIndex": index,
                "durationMs": fallback_duration_ms.max(16),
            })
        })
        .collect()
}

fn capture_probe_for_item_id<'a>(
    item_id: &str,
    captures_by_asset_id: &'a BTreeMap<String, CaptureProbe>,
    captures_by_variant_key: &'a BTreeMap<String, CaptureProbe>,
) -> Option<&'a CaptureProbe> {
    captures_by_asset_id
        .get(&format!("nesqlpp:item/{item_id}"))
        .or_else(|| captures_by_variant_key.get(item_id))
}

fn write_json_field_name<W: Write>(writer: &mut W, first: &mut bool, key: &str) -> Result<()> {
    if !*first {
        writer.write_all(b",")?;
    }
    *first = false;
    serde_json::to_writer(&mut *writer, key)?;
    writer.write_all(b":")?;
    Ok(())
}

fn write_json_field<W: Write, T: Serialize + ?Sized>(
    writer: &mut W,
    first: &mut bool,
    key: &str,
    value: &T,
) -> Result<()> {
    write_json_field_name(writer, first, key)?;
    serde_json::to_writer(writer, value)?;
    Ok(())
}

fn write_json_map_entry<W: Write>(
    writer: &mut W,
    first: &mut bool,
    key: &str,
    value: &Value,
) -> Result<()> {
    if !*first {
        writer.write_all(b",")?;
    }
    *first = false;
    serde_json::to_writer(&mut *writer, key)?;
    writer.write_all(b":")?;
    serde_json::to_writer(writer, value)?;
    Ok(())
}

fn compact_item_renderer_entry(row: &Value) -> Value {
    json!({
        "rendererClass": value_or_null(row, "rendererClass"),
        "rendererKind": value_or_null(row, "rendererKind"),
        "usesShader": value_bool(row, "usesShader"),
        "requiresFramebufferCapture": value_bool(row, "requiresFramebufferCapture"),
        "supportsNativeAtlas": value_bool(row, "supportsNativeAtlas"),
        "stackResolved": value_bool(row, "stackResolved"),
        "hasNbt": value_bool(row, "hasNbt"),
    })
}

fn compact_shader_item_entry(row: &Value) -> Value {
    json!({
        "rendererKind": value_or_null(row, "rendererKind"),
        "rendererClass": value_or_null(row, "rendererClass"),
        "shaderFamily": value_or_null(row, "shaderFamily"),
        "timeSource": value_or_null(row, "timeSource"),
        "captureRequired": value_bool(row, "captureRequired"),
        "preferredExport": value_or_null(row, "preferredExport"),
        "browserReimplementationAllowed": value_bool(row, "browserReimplementationAllowed"),
    })
}

fn compact_sprite_entry(row: &Value) -> Value {
    let frame_count = stable_value_u64(row.get("frameCount"), 1);
    let fallback_duration_ms = stable_value_u64(row.get("defaultFrameTimeTicks"), 1) * 50;
    json!({
        "atlas": value_or_null(row, "atlas"),
        "spriteKey": value_or_null(row, "spriteKey"),
        "iconName": value_or_null(row, "iconName"),
        "spriteClass": value_or_null(row, "spriteClass"),
        "originX": stable_value_i64(row.get("originX"), -1),
        "originY": stable_value_i64(row.get("originY"), -1),
        "width": stable_value_i64(row.get("width"), -1),
        "height": stable_value_i64(row.get("height"), -1),
        "animated": value_bool(row, "animated"),
        "frameCount": frame_count,
        "defaultFrameTimeTicks": value_or_null(row, "defaultFrameTimeTicks"),
        "metadataFrameCount": value_or_null(row, "metadataFrameCount"),
        "interpolate": value_bool(row, "interpolate"),
        "timeline": normalize_animation_timeline(row.get("timeline"), frame_count, fallback_duration_ms),
    })
}

fn capture_probe_from_compact(capture: &Value) -> CaptureProbe {
    CaptureProbe {
        frame_count: stable_value_u64(capture.get("frameCount"), 0),
        frames: capture
            .get("frames")
            .and_then(Value::as_array)
            .map(Vec::len)
            .unwrap_or(0),
        timeline: capture
            .get("timeline")
            .and_then(Value::as_array)
            .map(Vec::len)
            .unwrap_or(0),
    }
}

fn capture_sample_requirement(
    requirement: &ShaderCaptureRequirement,
    capture: Option<&CaptureProbe>,
) -> Value {
    json!({
        "itemId": requirement.item_id,
        "expectedAssetId": format!("nesqlpp:item/{}", requirement.item_id),
        "rendererKind": requirement.renderer_kind,
        "rendererClass": requirement.renderer_class,
        "shaderFamily": requirement.shader_family,
        "preferredExport": requirement.preferred_export,
        "capture": capture.map(|capture| json!({
            "frameCount": capture.frame_count,
            "frames": capture.frames,
            "timeline": capture.timeline,
        })).unwrap_or(Value::Null),
    })
}

fn native_render_validation(
    shader_requirements: &[ShaderCaptureRequirement],
    framebuffer_capture_count: usize,
    captures_by_asset_id: &BTreeMap<String, CaptureProbe>,
    captures_by_variant_key: &BTreeMap<String, CaptureProbe>,
    stats: &mut NativeRenderIndexStats,
) -> Value {
    let mut missing_required_captures = Vec::new();
    let mut required_captures_without_frames = Vec::new();
    let mut required_captures_without_timeline = Vec::new();
    for requirement in shader_requirements {
        let capture = capture_probe_for_item_id(
            &requirement.item_id,
            captures_by_asset_id,
            captures_by_variant_key,
        );
        match capture {
            None => missing_required_captures.push(requirement),
            Some(capture) if capture.frames == 0 => {
                required_captures_without_frames.push((requirement, capture));
            }
            Some(capture) if capture.frame_count > 1 && capture.timeline == 0 => {
                required_captures_without_timeline.push((requirement, capture));
            }
            _ => {}
        }
    }

    stats.shader_items_needing_capture = shader_requirements.len();
    stats.missing_required_captures = missing_required_captures.len();
    stats.required_captures_without_frames = required_captures_without_frames.len();
    stats.required_captures_without_timeline = required_captures_without_timeline.len();
    let capture_gate_ready = missing_required_captures.is_empty()
        && required_captures_without_frames.is_empty()
        && required_captures_without_timeline.is_empty();
    let summary = if shader_requirements.is_empty() {
        "No shader/custom renderer capture is required by the export.".to_string()
    } else {
        format!(
            "{} shader/custom renderer item(s) require capture; {} capture asset(s) compiled; missing={}; withoutFrames={}; withoutTimeline={}.",
            shader_requirements.len(),
            framebuffer_capture_count,
            missing_required_captures.len(),
            required_captures_without_frames.len(),
            required_captures_without_timeline.len()
        )
    };
    let sample_limit = 20usize;
    json!({
        "status": if shader_requirements.is_empty() || capture_gate_ready { "ready" } else { "blocked" },
        "shaderItemsNeedingCapture": shader_requirements.len(),
        "framebufferCaptures": framebuffer_capture_count,
        "missingRequiredCaptures": missing_required_captures.len(),
        "requiredCapturesWithoutFrames": required_captures_without_frames.len(),
        "requiredCapturesWithoutTimeline": required_captures_without_timeline.len(),
        "samples": {
            "missingRequiredCaptures": missing_required_captures
                .iter()
                .take(sample_limit)
                .map(|requirement| capture_sample_requirement(requirement, None))
                .collect::<Vec<_>>(),
            "requiredCapturesWithoutFrames": required_captures_without_frames
                .iter()
                .take(sample_limit)
                .map(|(requirement, capture)| capture_sample_requirement(requirement, Some(capture)))
                .collect::<Vec<_>>(),
            "requiredCapturesWithoutTimeline": required_captures_without_timeline
                .iter()
                .take(sample_limit)
                .map(|(requirement, capture)| capture_sample_requirement(requirement, Some(capture)))
                .collect::<Vec<_>>(),
        },
        "summary": summary,
    })
}

fn write_native_render_index(
    session: &RawExportSession,
    output: &Path,
) -> Result<NativeRenderIndexStats> {
    let native_render_path = output.join(NATIVE_RENDER_INDEX_PATH);
    if let Some(parent) = native_render_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = File::create(&native_render_path)
        .with_context(|| format!("create {}", native_render_path.display()))?;
    let mut writer = BufWriter::new(file);
    let backend = session
        .read_manifest_json("renderBackend")?
        .map(|value| value.as_ref().clone())
        .unwrap_or(Value::Null);
    let mut stats = NativeRenderIndexStats::default();

    writer.write_all(b"{")?;
    let mut field_first = true;
    write_json_field(
        &mut writer,
        &mut field_first,
        "schemaVersion",
        "neonei/native-render-index/v1",
    )?;
    write_json_field(&mut writer, &mut field_first, "backend", &backend)?;

    write_json_field_name(&mut writer, &mut field_first, "itemRendererByItemId")?;
    writer.write_all(b"{")?;
    let mut map_first = true;
    let mut item_renderer_keys = HashSet::<String>::new();
    let item_renderer_rows = session.read_manifest_jsonl("renderItemRenderers")?;
    stats.item_renderers = item_renderer_rows.len();
    for row in item_renderer_rows.iter() {
        let Some(item_id) = value_string(row, "itemId") else {
            continue;
        };
        if item_renderer_keys.insert(item_id.clone()) {
            write_json_map_entry(
                &mut writer,
                &mut map_first,
                &item_id,
                &compact_item_renderer_entry(row),
            )?;
        }
    }
    stats.item_renderer_by_item_id = item_renderer_keys.len();
    writer.write_all(b"}")?;

    write_json_field_name(&mut writer, &mut field_first, "shaderByItemId")?;
    writer.write_all(b"{")?;
    let mut map_first = true;
    let mut shader_keys = HashSet::<String>::new();
    let mut shader_requirements = Vec::<ShaderCaptureRequirement>::new();
    let shader_rows = session.read_manifest_jsonl("renderShaderItems")?;
    stats.shader_items = shader_rows.len();
    for row in shader_rows.iter() {
        let Some(item_id) = value_string(row, "itemId") else {
            continue;
        };
        if value_bool(row, "captureRequired") {
            shader_requirements.push(ShaderCaptureRequirement {
                item_id: item_id.clone(),
                renderer_kind: value_string(row, "rendererKind"),
                renderer_class: value_string(row, "rendererClass"),
                shader_family: value_string(row, "shaderFamily"),
                preferred_export: value_string(row, "preferredExport"),
            });
        }
        if shader_keys.insert(item_id.clone()) {
            write_json_map_entry(
                &mut writer,
                &mut map_first,
                &item_id,
                &compact_shader_item_entry(row),
            )?;
        }
    }
    stats.shader_by_item_id = shader_keys.len();
    writer.write_all(b"}")?;

    write_json_field_name(&mut writer, &mut field_first, "capturesByAssetId")?;
    writer.write_all(b"{")?;
    let capture_rows = session.read_manifest_jsonl("renderFramebufferCaptures")?;
    stats.framebuffer_captures = capture_rows.len();
    let (
        captures_by_asset_value,
        captures_by_variant_value,
        captures_by_asset_id,
        captures_by_variant_key,
    ) = index_framebuffer_captures(capture_rows.as_slice())?;
    let mut map_first = true;
    for (asset_id, compact) in &captures_by_asset_value {
        write_json_map_entry(&mut writer, &mut map_first, asset_id, compact)?;
    }
    writer.write_all(b"}")?;

    write_json_field_name(&mut writer, &mut field_first, "capturesByVariantKey")?;
    writer.write_all(b"{")?;
    let mut map_first = true;
    for (variant_key, compact) in &captures_by_variant_value {
        write_json_map_entry(&mut writer, &mut map_first, variant_key, compact)?;
    }
    writer.write_all(b"}")?;

    write_json_field_name(&mut writer, &mut field_first, "spriteByIconName")?;
    writer.write_all(b"{")?;
    let mut map_first = true;
    let mut sprite_keys = HashSet::<String>::new();
    let sprite_rows = session.read_manifest_jsonl("renderTextureSprites")?;
    stats.texture_sprites = sprite_rows.len();
    for row in sprite_rows.iter() {
        let Some(key) = value_string(row, "iconName").or_else(|| value_string(row, "spriteKey"))
        else {
            continue;
        };
        if sprite_keys.insert(key.clone()) {
            write_json_map_entry(
                &mut writer,
                &mut map_first,
                &key,
                &compact_sprite_entry(row),
            )?;
        }
    }
    stats.sprite_by_icon_name = sprite_keys.len();
    writer.write_all(b"}")?;

    let validation = native_render_validation(
        &shader_requirements,
        stats.framebuffer_captures,
        &captures_by_asset_id,
        &captures_by_variant_key,
        &mut stats,
    );
    write_json_field(
        &mut writer,
        &mut field_first,
        "counts",
        &json!({
            "textureSprites": stats.texture_sprites,
            "itemRenderers": stats.item_renderers,
            "shaderItems": stats.shader_items,
            "framebufferCaptures": stats.framebuffer_captures,
            "itemRendererByItemId": stats.item_renderer_by_item_id,
            "shaderByItemId": stats.shader_by_item_id,
            "spriteByIconName": stats.sprite_by_icon_name,
        }),
    )?;
    write_json_field(&mut writer, &mut field_first, "validation", &validation)?;
    writer.write_all(b"}\n")?;
    writer
        .flush()
        .with_context(|| format!("write {}", native_render_path.display()))?;
    Ok(stats)
}

fn copy_or_write_empty_native_render_index(
    session: &RawExportSession,
    output: &Path,
) -> Result<()> {
    let native_render_path = output.join(NATIVE_RENDER_INDEX_PATH);
    if let Some(parent) = native_render_path.parent() {
        fs::create_dir_all(parent)?;
    }
    if let Some(bytes) = session.read_optional_manifest_bytes("nativeRenderIndex")? {
        fs::write(&native_render_path, bytes.as_slice())
            .with_context(|| format!("write {}", native_render_path.display()))?;
        return Ok(());
    }
    let empty_index = json!({
        "schemaVersion": "neonei/native-render-index/v1",
        "backend": Value::Null,
        "counts": {
            "textureSprites": 0,
            "itemRenderers": 0,
            "shaderItems": 0,
            "framebufferCaptures": 0,
            "itemRendererByItemId": 0,
            "shaderByItemId": 0,
            "spriteByIconName": 0,
        },
        "itemRendererByItemId": {},
        "shaderByItemId": {},
        "capturesByAssetId": {},
        "capturesByVariantKey": {},
        "spriteByIconName": {},
        "validation": {
            "status": "ready",
            "shaderItemsNeedingCapture": 0,
            "framebufferCaptures": 0,
            "missingRequiredCaptures": 0,
            "requiredCapturesWithoutFrames": 0,
            "requiredCapturesWithoutTimeline": 0,
            "samples": {
                "missingRequiredCaptures": [],
                "requiredCapturesWithoutFrames": [],
                "requiredCapturesWithoutTimeline": [],
            },
            "summary": "No shader/custom renderer capture is required by the export.",
        },
    });
    write_json_value(&native_render_path, &empty_index)
}

pub fn compile_texture_pack(
    session: &RawExportSession,
    output: &Path,
    strict: bool,
    debug_json: bool,
) -> Result<()> {
    let manifest = session.manifest();
    if manifest.files.contains_key("textureManifest") {
        return compile_dist_texture_pack(session, output, strict, debug_json);
    }
    let atlas = session
        .read_manifest_json("browserAtlasIndex")?
        .ok_or_else(|| anyhow!("texture compiler blocked: browserAtlasIndex is missing"))?;
    let animations = session.read_manifest_collection(COLLECTION_ANIMATIONS)?;
    let native_sprites = session.read_manifest_collection(COLLECTION_NATIVE_SPRITES)?;
    let facade_resolutions = session.read_manifest_collection(COLLECTION_FACADE_RESOLUTIONS)?;
    let animation_materializations =
        session.read_manifest_collection(COLLECTION_ANIMATION_FRAME_MATERIALIZATIONS)?;
    let texture_rows = session.read_manifest_collection(COLLECTION_TEXTURE_ROWS_WITH_MANIFEST)?;
    let item_rows = session.read_manifest_collection(COLLECTION_BROWSER_ITEMS)?;
    let animation_count = animations.len();
    let native_sprite_count = native_sprites.len();
    let facade_resolution_count = facade_resolutions.len();
    let animation_materialization_count = animation_materializations.len();
    let texture_row_count = texture_rows.len();

    let animation_by_asset = animations
        .iter()
        .filter_map(|row| Some((value_string(row, "assetId")?, row.clone())))
        .collect::<BTreeMap<_, _>>();
    drop(animations);
    let native_sprite_by_asset = native_sprites
        .iter()
        .filter_map(|row| Some((value_string(row, "assetId")?, row.clone())))
        .collect::<BTreeMap<_, _>>();
    drop(native_sprites);
    let mut animation_materialization_by_asset = BTreeMap::new();
    let mut animation_materialization_index_issues = Vec::new();
    let mut invalid_animation_materialization_assets = HashSet::new();
    for row in animation_materializations.iter() {
        let Some(asset_id) = value_string(row, "assetId").filter(|value| !value.trim().is_empty())
        else {
            animation_materialization_index_issues.push(json!({
                "code": "ANIMATION_MATERIALIZATION_INVALID",
                "itemId": Value::Null,
                "assetId": Value::Null,
                "validationError": "animationFrameMaterializations row has no non-empty assetId",
            }));
            continue;
        };
        if invalid_animation_materialization_assets.contains(&asset_id) {
            continue;
        }
        if animation_materialization_by_asset.contains_key(&asset_id) {
            invalid_animation_materialization_assets.insert(asset_id.clone());
            animation_materialization_by_asset.remove(&asset_id);
            animation_materialization_index_issues.push(json!({
                "code": "ANIMATION_MATERIALIZATION_INVALID",
                "itemId": Value::Null,
                "assetId": asset_id,
                "validationError": "animationFrameMaterializations must contain exactly one row per assetId",
            }));
            continue;
        }
        if let Some((diagnostic, true)) = animation_materialization_diagnostic("", &asset_id, row) {
            invalid_animation_materialization_assets.insert(asset_id.clone());
            animation_materialization_index_issues.push(diagnostic);
        }
        animation_materialization_by_asset.insert(asset_id, row.clone());
    }
    drop(animation_materializations);
    let texture_by_asset = if debug_json {
        Some(
            texture_rows
                .iter()
                .filter_map(|row| Some((value_string(row, "assetId")?, row.clone())))
                .collect::<BTreeMap<_, _>>(),
        )
    } else {
        None
    };

    let atlas_repair =
        repaired_browser_atlas_items(&atlas, &texture_rows, &item_rows, &facade_resolutions);
    let repaired_facade_count = atlas_repair.repaired_facades;
    let unresolved_facade_count = atlas_repair.unresolved_facades.len();
    let mut atlas_items = atlas_repair.items;
    drop(item_rows);
    if !debug_json {
        drop(texture_rows);
    }
    promote_animation_facts_to_animated_atlas_items(
        &mut atlas_items,
        &animation_materialization_by_asset,
        &animation_by_asset,
        &native_sprite_by_asset,
    );
    let debug_atlas = if debug_json {
        Some(json!({
            "schemaVersion": atlas
                .get("schemaVersion")
                .cloned()
                .unwrap_or(Value::Null),
            "items": atlas_items.clone(),
        }))
    } else {
        None
    };

    let mut static_items = 0u64;
    let mut animated_items = 0u64;
    let mut missing_atlas_file_refs = Vec::new();
    let mut missing_atlas_asset_files = Vec::new();
    let mut invalid_atlas_bounds = Vec::new();
    let mut invalid_frame_bounds = Vec::new();
    let mut actionable_texture_issues = atlas_repair.unresolved_facades;
    actionable_texture_issues.extend(animation_materialization_index_issues);
    let mut animation_materialization_diagnostics = Vec::new();
    let mut animation_table = Vec::new();
    let mut atlas_map = if debug_json {
        Some(BTreeMap::new())
    } else {
        None
    };

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
        let animation_materialization = animation_materialization_by_asset.get(&asset_id);
        if !has_static_atlas && !has_animated_atlas {
            actionable_texture_issues.push(json!({
                "code": "TEXTURE_ATLAS_ENTRY_EMPTY",
                "itemId": item_id,
                "assetId": asset_id,
                "reason": "No staticAtlas or animatedAtlas was generated for this browser atlas item.",
                "recommendedFix": "Fix NESQL++ texture capture or atlas-source classification for this item; do not use frontend per-item image fallback.",
            }));
        }
        if let Some(materialization) = animation_materialization {
            if !invalid_animation_materialization_assets.contains(&asset_id) {
                if let Some((diagnostic, blocking)) =
                    animation_materialization_diagnostic(&item_id, &asset_id, materialization)
                {
                    if blocking {
                        actionable_texture_issues.push(diagnostic);
                    } else {
                        animation_materialization_diagnostics.push(diagnostic);
                    }
                }
            }
        }
        if !has_animated_atlas
            && expected_animated_item(animation, native_sprite)
            && animation_materialization.is_none()
        {
            actionable_texture_issues.push(json!({
                "code": "EXPECTED_ANIMATED_BUT_STATIC",
                "itemId": item_id,
                "assetId": asset_id,
                "reason": expected_animation_reason(animation, native_sprite),
                "hasStaticAtlas": has_static_atlas,
                "hasAnimationFacts": animation.is_some(),
                "hasNativeSpriteFacts": native_sprite.is_some(),
                "hasAnimationFrameMaterializationFacts": false,
                "recommendedFix": "NESQL++ must export an authoritative animationFrameMaterializations row; compiler promotion from timing metadata or duplicated static coordinates is forbidden.",
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
            let frame_duration_source = animated_atlas
                .and_then(|value| value.get("frameDurationSource"))
                .cloned()
                .unwrap_or_else(|| {
                    json!(if animation_materialization.is_some() {
                        "animation_frame_materialization"
                    } else if native_sprite.is_some() {
                        "native_sprite_metadata"
                    } else {
                        "raw_animation_index"
                    })
                });
            animation_table.push(json!({
                "itemId": item_id,
                "assetId": asset_id,
                "mode": animation_materialization
                    .and_then(|value| value.get("materializationStatus"))
                    .cloned()
                    .or_else(|| native_sprite.and_then(|value| value.get("animationMode")).cloned())
                    .unwrap_or(Value::Null),
                "frameDurationSource": frame_duration_source,
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
        if let Some(atlas_map) = atlas_map.as_mut() {
            let texture = texture_by_asset
                .as_ref()
                .and_then(|values| values.get(&asset_id))
                .cloned()
                .unwrap_or(Value::Null);
            atlas_map.insert(
                item_id,
                json!({
                    "assetId": asset_id,
                    "atlas": item,
                    "texture": texture,
                }),
            );
        }
    }
    copy_runtime_atlas_assets_from_session(
        session,
        output,
        &atlas_items,
        &mut missing_atlas_asset_files,
    )?;

    let rust_dir = output.join("rust");
    fs::create_dir_all(&rust_dir)?;
    let blocking_actionable_issue_count = actionable_texture_issues
        .iter()
        .filter(|issue| {
            matches!(
                value_string(issue, "code").as_deref(),
                Some(
                    "EXPECTED_ANIMATED_BUT_STATIC"
                        | "ANIMATION_MATERIALIZATION_INVALID"
                        | "FACADE_RESOLUTION_FACT_MISSING"
                        | "FACADE_RESOLUTION_FACT_INVALID"
                        | "FACADE_RESOLUTION_UNRESOLVED"
                        | "FACADE_RESOLUTION_SOURCE_MISSING"
                )
            )
        })
        .count();
    let certified_static_animation_count = animation_materialization_diagnostics
        .iter()
        .filter(|diagnostic| {
            value_string(diagnostic, "code").as_deref() == Some("ANIMATION_CERTIFIED_STATIC")
        })
        .count();
    let certified_unavailable_animation_count = animation_materialization_diagnostics
        .iter()
        .filter(|diagnostic| {
            value_string(diagnostic, "code").as_deref() == Some("ANIMATION_CERTIFIED_UNAVAILABLE")
        })
        .count();
    let invalid_animation_materialization_count = actionable_texture_issues
        .iter()
        .filter(|issue| {
            value_string(issue, "code").as_deref() == Some("ANIMATION_MATERIALIZATION_INVALID")
        })
        .count();
    let missing_animation_materialization_count = actionable_texture_issues
        .iter()
        .filter(|issue| {
            value_string(issue, "code").as_deref() == Some("EXPECTED_ANIMATED_BUT_STATIC")
        })
        .count();
    let missing_atlas_file_ref_count = missing_atlas_file_refs.len();
    let missing_atlas_asset_file_count = missing_atlas_asset_files.len();
    let invalid_atlas_bound_count = invalid_atlas_bounds.len();
    let invalid_frame_bound_count = invalid_frame_bounds.len();
    let blocking_texture_integrity_issue_count = missing_atlas_file_ref_count
        + missing_atlas_asset_file_count
        + invalid_atlas_bound_count
        + invalid_frame_bound_count;
    let strict_texture_blocked =
        blocking_actionable_issue_count > 0 || blocking_texture_integrity_issue_count > 0;
    let texture_report_status = if strict_texture_blocked {
        "blocked"
    } else if actionable_texture_issues.is_empty()
        && animation_materialization_diagnostics.is_empty()
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
            "blockingActionableIssues": blocking_actionable_issue_count,
            "repairedBuildCraftFacades": repaired_facade_count,
            "unresolvedBuildCraftFacades": unresolved_facade_count,
            "facadeResolutionRows": facade_resolution_count,
            "animationFrameMaterializationRows": animation_materialization_count,
            "certifiedStaticAnimations": certified_static_animation_count,
            "certifiedUnavailableAnimations": certified_unavailable_animation_count,
            "invalidAnimationMaterializations": invalid_animation_materialization_count,
            "missingAnimationMaterializations": missing_animation_materialization_count,
            "missingAtlasFileRefs": missing_atlas_file_refs.len(),
            "missingAtlasAssetFiles": missing_atlas_asset_files.len(),
            "invalidAtlasBounds": invalid_atlas_bounds.len(),
            "invalidFrameBounds": invalid_frame_bounds.len(),
        },
        "issues": actionable_texture_issues,
        "animationMaterializationDiagnostics": animation_materialization_diagnostics,
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
                "blockingActionableIssues": suspicious_texture_report["counts"]["blockingActionableIssues"].clone(),
                "repairedBuildCraftFacades": suspicious_texture_report["counts"]["repairedBuildCraftFacades"].clone(),
                "unresolvedBuildCraftFacades": suspicious_texture_report["counts"]["unresolvedBuildCraftFacades"].clone(),
                "facadeResolutionRows": suspicious_texture_report["counts"]["facadeResolutionRows"].clone(),
                "animationFrameMaterializationRows": suspicious_texture_report["counts"]["animationFrameMaterializationRows"].clone(),
                "certifiedStaticAnimations": suspicious_texture_report["counts"]["certifiedStaticAnimations"].clone(),
                "certifiedUnavailableAnimations": suspicious_texture_report["counts"]["certifiedUnavailableAnimations"].clone(),
                "invalidAnimationMaterializations": suspicious_texture_report["counts"]["invalidAnimationMaterializations"].clone(),
                "missingAnimationMaterializations": suspicious_texture_report["counts"]["missingAnimationMaterializations"].clone(),
                "missingAtlasFileRefs": suspicious_texture_report["counts"]["missingAtlasFileRefs"].clone(),
                "missingAtlasAssetFiles": suspicious_texture_report["counts"]["missingAtlasAssetFiles"].clone(),
                "invalidAtlasBounds": suspicious_texture_report["counts"]["invalidAtlasBounds"].clone(),
                "invalidFrameBounds": suspicious_texture_report["counts"]["invalidFrameBounds"].clone(),
            },
            "issues": suspicious_texture_report["issues"].clone(),
            "animationMaterializationDiagnostics": suspicious_texture_report["animationMaterializationDiagnostics"].clone(),
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
    if strict && strict_texture_blocked {
        return Err(anyhow!(
            "texture compiler blocked: blocking actionable issues={}, missing atlas refs={}, missing atlas assets={}, invalid atlas bounds={}, invalid frame bounds={}; diagnostics were written to rust/missing-texture-report.json and rust/suspicious-texture-report.json",
            blocking_actionable_issue_count,
            missing_atlas_file_ref_count,
            missing_atlas_asset_file_count,
            invalid_atlas_bound_count,
            invalid_frame_bound_count
        ));
    }
    let native_render_stats = write_native_render_index(session, output)?;
    if debug_json {
        let atlas_map = atlas_map.unwrap_or_default();
        let texture_output_pack = json!({
            "schemaVersion": "neonei/rust-texture-pack/current",
            "counts": {
                "atlasItems": atlas_items.len(),
                "staticAtlasItems": static_items,
                "animatedAtlasItems": animated_items,
                "animationRows": animation_count,
                "nativeSpriteRows": native_sprite_count,
                "textureRows": texture_row_count,
                "facadeResolutionRows": facade_resolution_count,
                "animationFrameMaterializationRows": animation_materialization_count,
                "certifiedStaticAnimations": certified_static_animation_count,
                "certifiedUnavailableAnimations": certified_unavailable_animation_count,
                "invalidAnimationMaterializations": invalid_animation_materialization_count,
                "missingAnimationMaterializations": missing_animation_materialization_count,
                "blockingActionableIssues": suspicious_texture_report["counts"]["blockingActionableIssues"].clone(),
                "repairedBuildCraftFacades": suspicious_texture_report["counts"]["repairedBuildCraftFacades"].clone(),
                "unresolvedBuildCraftFacades": suspicious_texture_report["counts"]["unresolvedBuildCraftFacades"].clone(),
                "missingAtlasFileRefs": suspicious_texture_report["counts"]["missingAtlasFileRefs"].clone(),
                "missingAtlasAssetFiles": suspicious_texture_report["counts"]["missingAtlasAssetFiles"].clone(),
                "invalidAtlasBounds": suspicious_texture_report["counts"]["invalidAtlasBounds"].clone(),
                "invalidFrameBounds": suspicious_texture_report["counts"]["invalidFrameBounds"].clone(),
                "atlasMapItems": atlas_map.len(),
                "renderTextureSprites": native_render_stats.texture_sprites,
                "renderItemRenderers": native_render_stats.item_renderers,
                "renderShaderItems": native_render_stats.shader_items,
                "renderFramebufferCaptures": native_render_stats.framebuffer_captures,
            },
            "atlas": debug_atlas.as_ref().unwrap_or(&Value::Null),
            "atlasMap": atlas_map,
            "animationTable": &animation_table,
            "nativeRenderIndexPath": NATIVE_RENDER_INDEX_PATH,
            "validation": {
                "missingAtlasFileRefs": suspicious_texture_report["missingAtlasFileRefs"].clone(),
                "missingAtlasAssetFiles": suspicious_texture_report["missingAtlasAssetFiles"].clone(),
                "invalidAtlasBounds": suspicious_texture_report["invalidAtlasBounds"].clone(),
                "invalidFrameBounds": suspicious_texture_report["invalidFrameBounds"].clone(),
            },
        });
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
    session: &RawExportSession,
    output: &Path,
    strict: bool,
    debug_json: bool,
) -> Result<()> {
    let texture_files = session.runtime_file_descriptors(&[
        ("textureManifest", "textureManifest"),
        ("browserAtlasIndex", "browserAtlasIndex"),
        ("nativeRenderIndex", "nativeRenderIndex"),
        ("animationTable", "animationTable"),
        ("animationExpectationReport", "animationExpectationReport"),
    ])?;
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
        "files": session.runtime_file_descriptors(
            &[("animationTable", "animationTable"), ("animationExpectationReport", "animationExpectationReport")],
        )?,
    });

    let rust_dir = output.join("rust");
    fs::create_dir_all(&rust_dir)?;
    copy_or_write_empty_native_render_index(session, output)?;
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

    let mut sorted_animations = animation_table.iter().collect::<Vec<_>>();
    sorted_animations.sort_by_key(|left| value_string(left, "itemId"));

    for animation in sorted_animations {
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

    let mut sorted_items = atlas_items.iter().collect::<Vec<_>>();
    sorted_items.sort_by_key(|left| value_string(left, "itemId"));

    for item in sorted_items {
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
            let timeline = normalize_timeline(
                Some(animated_atlas),
                optional_value_u64(Some(animated_atlas), "frameDurationMs"),
            );
            let timeline_values = timeline.as_array();
            let frame_values = animated_atlas
                .get("frames")
                .and_then(Value::as_array)
                .into_iter()
                .flatten();
            for (index, frame) in frame_values.enumerate() {
                let duration_ms = timeline_values
                    .and_then(|values| values.get(index))
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

#[cfg(test)]
mod framebuffer_capture_index_tests {
    use super::*;

    #[test]
    fn duplicate_asset_id_is_rejected_instead_of_splitting_validation_and_output() {
        let rows = vec![
            json!({"assetId":"asset:duplicate","variantKey":"variant:first","frames":[]}),
            json!({"assetId":"asset:duplicate","variantKey":"variant:second","frames":[{"textureKey":"texture:ok"}]}),
        ];
        let error = index_framebuffer_captures(&rows).unwrap_err();
        assert!(error
            .to_string()
            .contains("duplicate framebuffer capture assetId"));
    }

    #[test]
    fn conflicting_variant_key_is_rejected_instead_of_splitting_validation_and_output() {
        let rows = vec![
            json!({"assetId":"asset:first","variantKey":"variant:duplicate","timeline":[]}),
            json!({"assetId":"asset:second","variantKey":"variant:duplicate","timeline":[{"frameIndex":0,"durationMs":50}]}),
        ];
        let error = index_framebuffer_captures(&rows).unwrap_err();
        assert!(error
            .to_string()
            .contains("conflicting framebuffer capture variantKey"));
    }

    #[test]
    fn equivalent_variant_key_aliases_share_one_variant_capture() {
        let rows = vec![
            json!({
                "assetId":"asset:legacy",
                "variantKey":"variant:shared",
                "primaryArtifact":"image/shared.gif",
                "frames":[{"path":"image/shared.gif","index":0}]
            }),
            json!({
                "assetId":"asset:variant",
                "variantKey":"variant:shared",
                "primaryArtifact":"image/shared.gif",
                "frames":[{"path":"image/shared.gif","index":0}]
            }),
        ];
        let (by_asset, by_variant, probes_by_asset, probes_by_variant) =
            index_framebuffer_captures(&rows).unwrap();
        assert_eq!(by_asset.len(), 2);
        assert_eq!(probes_by_asset.len(), 2);
        assert_eq!(by_variant.len(), 1);
        assert_eq!(probes_by_variant.len(), 1);
    }
}
