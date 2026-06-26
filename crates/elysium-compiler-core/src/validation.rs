use crate::io::write_json_value;
use crate::json_ext::{numeric_value_u64, value_u64};
use crate::manifest::{read_manifest, read_manifest_json};
use anyhow::Result;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

pub fn compile_semantic_validation_report(input: &Path, output: &Path) -> Result<()> {
    let manifest = read_manifest(input)?;
    let export_report =
        read_manifest_json(input, &manifest, "exportReport")?.unwrap_or(Value::Null);
    let browser_contract =
        read_manifest_json(input, &manifest, "neiBrowserContract")?.unwrap_or(Value::Null);
    let family_audit =
        read_manifest_json(input, &manifest, "semanticFamilyAudit")?.unwrap_or(Value::Null);
    let nbt_distribution =
        read_manifest_json(input, &manifest, "semanticNbtKeyDistribution")?.unwrap_or(Value::Null);
    let identity_report =
        read_manifest_json(input, &manifest, "semanticIdentityNormalizationReport")?
            .unwrap_or(Value::Null);

    let export_counts = export_report
        .get("counts")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let browser_status = browser_contract
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("missing");
    let representative_mismatches =
        value_u64(&browser_contract, "representativeMismatchCount").unwrap_or(0);
    let missing_representatives =
        value_u64(&browser_contract, "missingRepresentativeCount").unwrap_or(0);
    let fallback_groups = value_u64(&browser_contract, "fallbackGroupCount").unwrap_or(0);
    let native_groups = value_u64(&browser_contract, "nativeGroupCount").unwrap_or(0);
    let semantic_unclassified =
        value_u64(&export_counts, "semanticUnclassifiedTaggedItems").unwrap_or(0);
    let render_shader_missing =
        value_u64(&export_counts, "renderShaderItemsMissingCapture").unwrap_or(0);
    let render_texture_missing_timing =
        value_u64(&export_counts, "renderTextureSpritesMissingTiming").unwrap_or(0);

    let mut warnings = Vec::new();
    if browser_status != "ok" {
        warnings.push(json!({
            "code": "NEI_BROWSER_CONTRACT_NOT_OK",
            "status": browser_status,
        }));
    }
    if fallback_groups > 0 && native_groups == 0 {
        warnings.push(json!({
            "code": "NEI_GROUPS_FALLBACK_ONLY",
            "fallbackGroups": fallback_groups,
            "nativeGroups": native_groups,
            "message": "Raw export has collapsible group data, but no native group rows were identified.",
        }));
    }
    if semantic_unclassified > 0 {
        warnings.push(json!({
            "code": "SEMANTIC_TAGGED_ITEMS_UNCLASSIFIED",
            "count": semantic_unclassified,
        }));
    }
    if render_shader_missing > 0 {
        warnings.push(json!({
            "code": "RENDER_SHADER_CAPTURE_MISSING",
            "count": render_shader_missing,
            "samples": export_counts
                .get("renderShaderItemsMissingCaptureSamples")
                .cloned()
                .unwrap_or(Value::Null),
        }));
    }
    if render_texture_missing_timing > 0 {
        warnings.push(json!({
            "code": "RENDER_TEXTURE_TIMING_MISSING",
            "count": render_texture_missing_timing,
        }));
    }
    let blocking_count = representative_mismatches + missing_representatives;
    let status = if blocking_count > 0 {
        "blocked"
    } else if warnings.is_empty() {
        "ok"
    } else {
        "advisory"
    };

    let rust_dir = output.join("rust");
    fs::create_dir_all(&rust_dir)?;
    write_json_value(
        &rust_dir.join("semantic-validation-report.json"),
        &json!({
            "schemaVersion": "neonei/rust-semantic-validation-report/current",
            "generatedAt": "deterministic-rust-compiler",
            "status": status,
            "source": {
                "exportReport": manifest.files.get("exportReport").cloned().unwrap_or_default(),
                "neiBrowserContract": manifest.files.get("neiBrowserContract").cloned().unwrap_or_default(),
                "semanticFamilyAudit": manifest.files.get("semanticFamilyAudit").cloned().unwrap_or_default(),
                "semanticNbtKeyDistribution": manifest.files.get("semanticNbtKeyDistribution").cloned().unwrap_or_default(),
                "semanticIdentityNormalizationReport": manifest.files.get("semanticIdentityNormalizationReport").cloned().unwrap_or_default(),
            },
            "counts": {
                "items": value_u64(&export_counts, "items").unwrap_or(0),
                "recipes": value_u64(&export_counts, "recipes").unwrap_or(0),
                "semanticTotalItems": value_u64(&export_counts, "semanticTotalItems").unwrap_or(0),
                "semanticTaggedItems": value_u64(&export_counts, "semanticTaggedItems").unwrap_or(0),
                "semanticClassifiedTaggedItems": value_u64(&export_counts, "semanticClassifiedTaggedItems").unwrap_or(0),
                "semanticUnclassifiedTaggedItems": semantic_unclassified,
                "semanticEstimatedPublicItems": value_u64(&export_counts, "semanticEstimatedPublicItems").unwrap_or(0),
                "semanticFamilyCount": value_u64(&export_counts, "semanticFamilyCount").unwrap_or(0),
                "browserItemCount": value_u64(&browser_contract, "browserItemCount").unwrap_or(0),
                "defaultEntryCount": value_u64(&browser_contract, "defaultEntryCount").unwrap_or(0),
                "groupCount": value_u64(&browser_contract, "groupCount").unwrap_or(0),
                "nativeGroupCount": native_groups,
                "fallbackGroupCount": fallback_groups,
                "hiddenItemCount": value_u64(&browser_contract, "hiddenItemCount").unwrap_or(0),
                "representativeMismatchCount": representative_mismatches,
                "missingRepresentativeCount": missing_representatives,
                "renderShaderItemsMissingCapture": render_shader_missing,
                "renderTextureSpritesMissingTiming": render_texture_missing_timing,
            },
            "semanticReports": {
                "familyAuditStatus": family_audit.get("status").cloned().unwrap_or(Value::Null),
                "nbtKeyDistributionStatus": nbt_distribution.get("status").cloned().unwrap_or(Value::Null),
                "identityNormalizationStatus": identity_report.get("status").cloned().unwrap_or(Value::Null),
            },
            "warnings": warnings,
            "blocking": {
                "representativeMismatchCount": representative_mismatches,
                "missingRepresentativeCount": missing_representatives,
            },
        }),
    )
}

pub fn validate_atlas_ref(item_id: &str, atlas: Option<&Value>, missing_refs: &mut Vec<String>) {
    let Some(atlas) = atlas else {
        missing_refs.push(format!("{item_id}:missing-atlas-object"));
        return;
    };
    if atlas
        .get("atlasFile")
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or_default()
        .is_empty()
    {
        missing_refs.push(format!("{item_id}:missing-atlas-file"));
    }
}

pub fn validate_atlas_bounds(
    item_id: &str,
    atlas_kind: &str,
    atlas: Option<&Value>,
    invalid_bounds: &mut Vec<String>,
) {
    let Some(atlas) = atlas else {
        return;
    };
    let has_rect = ["x", "y", "width", "height"]
        .iter()
        .any(|key| atlas.get(*key).is_some());
    if !has_rect && atlas_kind == "animated" {
        return;
    }
    let atlas_width = value_u64(atlas, "atlasWidth").unwrap_or(0);
    let atlas_height = value_u64(atlas, "atlasHeight").unwrap_or(0);
    let x = value_u64(atlas, "x").unwrap_or(0);
    let y = value_u64(atlas, "y").unwrap_or(0);
    let width = value_u64(atlas, "width").unwrap_or(0);
    let height = value_u64(atlas, "height").unwrap_or(0);
    if width == 0
        || height == 0
        || (atlas_width > 0 && x.saturating_add(width) > atlas_width)
        || (atlas_height > 0 && y.saturating_add(height) > atlas_height)
    {
        invalid_bounds.push(format!("{item_id}:{atlas_kind}:out-of-bounds"));
    }
}

pub fn validate_frame_bounds(
    item_id: &str,
    atlas: Option<&Value>,
    invalid_bounds: &mut Vec<String>,
) {
    let Some(atlas) = atlas else {
        return;
    };
    let atlas_width = value_u64(atlas, "atlasWidth").unwrap_or(0);
    let atlas_height = value_u64(atlas, "atlasHeight").unwrap_or(0);
    let Some(frames) = atlas.get("frames").and_then(Value::as_array) else {
        return;
    };
    for (index, frame) in frames.iter().enumerate() {
        let bounds = if let Some(values) = frame.as_array() {
            if values.len() < 5 {
                invalid_bounds.push(format!("{item_id}:frame-{index}:short"));
                continue;
            }
            Some((
                values.get(1).and_then(numeric_value_u64).unwrap_or(0),
                values.get(2).and_then(numeric_value_u64).unwrap_or(0),
                values.get(3).and_then(numeric_value_u64).unwrap_or(0),
                values.get(4).and_then(numeric_value_u64).unwrap_or(0),
            ))
        } else if frame.is_object() {
            Some((
                value_u64(frame, "x").unwrap_or(0),
                value_u64(frame, "y").unwrap_or(0),
                value_u64(frame, "width").unwrap_or(0),
                value_u64(frame, "height").unwrap_or(0),
            ))
        } else {
            invalid_bounds.push(format!("{item_id}:frame-{index}:unsupported-shape"));
            None
        };
        let Some((x, y, width, height)) = bounds else {
            continue;
        };
        if width == 0
            || height == 0
            || (atlas_width > 0 && x + width > atlas_width)
            || (atlas_height > 0 && y + height > atlas_height)
        {
            invalid_bounds.push(format!("{item_id}:frame-{index}:out-of-bounds"));
        }
    }
}
