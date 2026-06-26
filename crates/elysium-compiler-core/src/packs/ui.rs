use crate::binary::{intern_compact_string, push_i32, push_u32, write_binary_pack_payload};
use crate::io::write_json_value;
use crate::json_ext::{value_i64, value_string, value_u64};
use crate::manifest::{read_manifest, read_manifest_json};
use crate::recipe_ui_payload::{
    build_raw_recipe_ui_payload_index, read_compiled_recipe_ui_payload_index,
};
use crate::ui_templates::{
    build_ui_assets_manifest, build_ui_family_census_report,
    build_ui_template_binding_index_report, build_ui_template_bindings,
    build_ui_template_catalog_report, materialize_ui_background_assets,
    ui_template_catalog_templates, ui_template_rect_action_count, ui_template_rect_count,
    ui_template_slot_count, ui_template_text_count,
};
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn compile_ui_pack(input: &Path, output: &Path, strict: bool, _debug_json: bool) -> Result<()> {
    let manifest = read_manifest(input)?;
    let template_catalog = read_manifest_json(input, &manifest, "uiTemplateCatalog")?;
    let Some(template_catalog) = template_catalog else {
        if strict {
            return Err(anyhow!(
                "ui-pack compiler blocked: uiTemplateCatalog is missing"
            ));
        }
        return Ok(());
    };
    let templates = ui_template_catalog_templates(&template_catalog);
    if strict && templates.is_empty() {
        return Err(anyhow!(
            "ui-pack compiler blocked: uiTemplateCatalog has no templates"
        ));
    }

    let recipe_ui_index = match read_compiled_recipe_ui_payload_index(output)? {
        Some(entries) => entries,
        None => build_raw_recipe_ui_payload_index(input, &manifest)?,
    };
    if strict && recipe_ui_index.is_empty() {
        return Err(anyhow!(
            "ui-pack compiler blocked: no recipe UI payload index entries are available"
        ));
    }

    let bindings = build_ui_template_bindings(&recipe_ui_index, &templates);
    let bound_recipe_count = bindings
        .iter()
        .filter(|entry| value_string(entry, "templateKey").is_some_and(|value| !value.is_empty()))
        .count();
    if strict && !recipe_ui_index.is_empty() && bound_recipe_count == 0 {
        return Err(anyhow!(
            "ui-pack compiler blocked: no recipe bindings matched a captured UI template"
        ));
    }

    let ui_pack_dir = output.join("rust").join("ui-pack");
    fs::create_dir_all(&ui_pack_dir)?;

    let mut strings = vec![String::new()];
    let mut string_refs = HashMap::new();
    string_refs.insert(String::new(), 0u32);
    let template_payload =
        build_compact_ui_template_payload(&templates, &mut strings, &mut string_refs)?;
    let binding_payload =
        build_compact_ui_binding_payload(&bindings, &mut strings, &mut string_refs)?;
    let string_payload = build_compact_ui_string_payload(&strings)?;
    let assets_manifest = build_ui_assets_manifest(&templates);
    let template_catalog_report = build_ui_template_catalog_report(
        &manifest
            .files
            .get("uiTemplateCatalog")
            .cloned()
            .unwrap_or_default(),
        &templates,
    );
    let binding_index_report = build_ui_template_binding_index_report(&bindings, &templates);
    let family_census_report = build_ui_family_census_report(&templates);
    let ui_background_assets = materialize_ui_background_assets(input, output, &assets_manifest)?;
    let missing_ui_background_assets = ui_background_assets
        .get("missing")
        .and_then(Value::as_array)
        .map(|items| items.len())
        .unwrap_or(0);
    if strict && missing_ui_background_assets > 0 {
        return Err(anyhow!(
            "ui-pack compiler blocked: {missing_ui_background_assets} native UI background asset(s) are missing"
        ));
    }
    let unbound_recipes = bindings
        .iter()
        .filter(|entry| {
            value_string(entry, "templateKey")
                .unwrap_or_default()
                .is_empty()
        })
        .take(100)
        .map(|entry| {
            json!({
                "recipeId": value_string(entry, "recipeId").unwrap_or_default(),
                "familyKey": value_string(entry, "familyKey").unwrap_or_default(),
                "recipeType": value_string(entry, "recipeType").unwrap_or_default(),
            })
        })
        .collect::<Vec<_>>();
    let status = if recipe_ui_index.is_empty() {
        "empty"
    } else if bound_recipe_count == bindings.len() {
        "ready"
    } else {
        "partial"
    };

    write_binary_pack_payload(
        &ui_pack_dir.join("ui_templates.bin"),
        "neonei/ui-template-pack/current",
        &template_payload,
    )?;
    write_binary_pack_payload(
        &ui_pack_dir.join("ui_bindings.bin"),
        "neonei/ui-binding-pack/current",
        &binding_payload,
    )?;
    write_binary_pack_payload(
        &ui_pack_dir.join("ui_strings.bin"),
        "neonei/ui-string-pack/current",
        &string_payload,
    )?;
    write_json_value(
        &ui_pack_dir.join("ui_assets.manifest.json"),
        &assets_manifest,
    )?;
    write_json_value(
        &ui_pack_dir.join("ui_template_catalog.json"),
        &template_catalog_report,
    )?;
    write_json_value(
        &ui_pack_dir.join("ui_template_binding_index.json"),
        &binding_index_report,
    )?;
    write_json_value(
        &ui_pack_dir.join("ui_family_census.json"),
        &family_census_report,
    )?;
    write_json_value(
        &ui_pack_dir.join("ui_pack_report.json"),
        &json!({
            "schemaVersion": "neonei/ui-pack-report/current",
            "generatedAt": "deterministic-rust-compiler",
            "status": status,
            "source": {
                "uiTemplateCatalog": manifest.files.get("uiTemplateCatalog").cloned().unwrap_or_default(),
                "recipeUiPayloadIndex": "recipes/ui-payload-index.json",
            },
            "summary": {
                "templateCount": templates.len(),
                "bindingCount": bindings.len(),
                "boundRecipeCount": bound_recipe_count,
                "unboundRecipeCount": bindings.len().saturating_sub(bound_recipe_count),
                "slotCount": templates.iter().map(ui_template_slot_count).sum::<usize>(),
                "textOverlayCount": templates.iter().map(ui_template_text_count).sum::<usize>(),
                "hotspotCount": templates.iter().map(|template| ui_template_rect_count(template, "hotspots")).sum::<usize>(),
                "viewportCount": templates.iter().map(|template| ui_template_rect_count(template, "viewports")).sum::<usize>(),
                "hotspotActionCount": templates.iter().map(|template| ui_template_rect_action_count(template, "hotspots")).sum::<usize>(),
                "viewportActionCount": templates.iter().map(|template| ui_template_rect_action_count(template, "viewports")).sum::<usize>(),
                "stringCount": strings.len(),
                "assetCount": assets_manifest.get("assets").and_then(Value::as_array).map(|items| items.len()).unwrap_or(0),
            },
            "format": {
                "templatePackMagic": "NEIUIT1_NUL",
                "templatePackVersion": 3,
                "templateStride": 19,
                "slotStride": 6,
                "textStride": 5,
                "rectStride": 12,
                "hotspotActionFields": true,
                "hotspotActionFieldNames": ["action", "itemId", "payloadKey"],
            },
            "artifacts": {
                "uiTemplates": "rust/ui-pack/ui_templates.bin",
                "uiBindings": "rust/ui-pack/ui_bindings.bin",
                "uiStrings": "rust/ui-pack/ui_strings.bin",
                "uiAssetsManifest": "rust/ui-pack/ui_assets.manifest.json",
            },
            "assets": {
                "uiBackgrounds": ui_background_assets,
            },
            "unboundRecipes": unbound_recipes,
        }),
    )?;
    Ok(())
}

pub fn build_compact_ui_template_payload(
    templates: &[Value],
    strings: &mut Vec<String>,
    string_refs: &mut HashMap<String, u32>,
) -> Result<Vec<u8>> {
    let mut template_bytes = Vec::new();
    let mut slot_bytes = Vec::new();
    let mut text_bytes = Vec::new();
    let mut hotspot_bytes = Vec::new();
    let mut viewport_bytes = Vec::new();
    let mut slot_count = 0u32;
    let mut text_count = 0u32;
    let mut hotspot_count = 0u32;
    let mut viewport_count = 0u32;

    for template in templates {
        let slot_start = slot_count;
        if let Some(slots) = template.get("slots").and_then(Value::as_array) {
            for slot in slots {
                push_u32(
                    &mut slot_bytes,
                    intern_compact_string(strings, string_refs, value_string(slot, "role")),
                );
                push_u32(
                    &mut slot_bytes,
                    value_u64(slot, "startIndex").unwrap_or(0) as u32,
                );
                push_u32(
                    &mut slot_bytes,
                    value_u64(slot, "columns").unwrap_or(0) as u32,
                );
                push_u32(&mut slot_bytes, value_u64(slot, "rows").unwrap_or(0) as u32);
                push_i32(&mut slot_bytes, value_i64(slot, "x").unwrap_or(0) as i32);
                push_i32(&mut slot_bytes, value_i64(slot, "y").unwrap_or(0) as i32);
                slot_count += 1;
            }
        }
        let slot_total = slot_count.saturating_sub(slot_start);
        let text_start = text_count;
        if let Some(overlays) = template.get("textOverlays").and_then(Value::as_array) {
            for overlay in overlays {
                push_u32(
                    &mut text_bytes,
                    intern_compact_string(strings, string_refs, value_string(overlay, "text")),
                );
                push_i32(&mut text_bytes, value_i64(overlay, "x").unwrap_or(0) as i32);
                push_i32(&mut text_bytes, value_i64(overlay, "y").unwrap_or(0) as i32);
                push_u32(
                    &mut text_bytes,
                    value_u64(overlay, "width").unwrap_or(0) as u32,
                );
                push_u32(
                    &mut text_bytes,
                    value_u64(overlay, "height").unwrap_or(0) as u32,
                );
                text_count += 1;
            }
        }
        let text_total = text_count.saturating_sub(text_start);
        let hotspot_start = hotspot_count;
        if let Some(hotspots) = template.get("hotspots").and_then(Value::as_array) {
            for hotspot in hotspots {
                push_compact_ui_rect(&mut hotspot_bytes, strings, string_refs, hotspot);
                hotspot_count += 1;
            }
        }
        let hotspot_total = hotspot_count.saturating_sub(hotspot_start);
        let viewport_start = viewport_count;
        if let Some(viewports) = template.get("viewports").and_then(Value::as_array) {
            for viewport in viewports {
                push_compact_ui_rect(&mut viewport_bytes, strings, string_refs, viewport);
                viewport_count += 1;
            }
        }
        let viewport_total = viewport_count.saturating_sub(viewport_start);

        push_u32(
            &mut template_bytes,
            intern_compact_string(strings, string_refs, value_string(template, "templateKey")),
        );
        push_u32(
            &mut template_bytes,
            intern_compact_string(
                strings,
                string_refs,
                value_string(template, "templateSignature"),
            ),
        );
        push_u32(
            &mut template_bytes,
            intern_compact_string(strings, string_refs, value_string(template, "familyKey")),
        );
        push_u32(
            &mut template_bytes,
            intern_compact_string(
                strings,
                string_refs,
                value_string(template, "canonicalMachineFamily"),
            ),
        );
        push_u32(
            &mut template_bytes,
            intern_compact_string(strings, string_refs, value_string(template, "layoutKind")),
        );
        push_u32(
            &mut template_bytes,
            value_u64(template, "width").unwrap_or(0) as u32,
        );
        push_u32(
            &mut template_bytes,
            value_u64(template, "height").unwrap_or(0) as u32,
        );
        push_i32(
            &mut template_bytes,
            value_i64(template, "yShift").unwrap_or(0) as i32,
        );
        push_u32(
            &mut template_bytes,
            value_u64(template, "maxRecipesPerPage").unwrap_or(1) as u32,
        );
        push_u32(
            &mut template_bytes,
            intern_compact_string(
                strings,
                string_refs,
                value_string(template, "imageResource"),
            ),
        );
        push_u32(
            &mut template_bytes,
            value_u64(template, "handlerCount").unwrap_or(0) as u32,
        );
        push_u32(&mut template_bytes, slot_start);
        push_u32(&mut template_bytes, slot_total);
        push_u32(&mut template_bytes, text_start);
        push_u32(&mut template_bytes, text_total);
        push_u32(&mut template_bytes, hotspot_start);
        push_u32(&mut template_bytes, hotspot_total);
        push_u32(&mut template_bytes, viewport_start);
        push_u32(&mut template_bytes, viewport_total);
    }

    let mut payload = Vec::with_capacity(
        8 + 10 * 4
            + template_bytes.len()
            + slot_bytes.len()
            + text_bytes.len()
            + hotspot_bytes.len()
            + viewport_bytes.len(),
    );
    payload.extend_from_slice(b"NEIUIT1\0");
    push_u32(&mut payload, 3);
    push_u32(&mut payload, templates.len() as u32);
    push_u32(&mut payload, slot_count);
    push_u32(&mut payload, text_count);
    push_u32(&mut payload, hotspot_count);
    push_u32(&mut payload, viewport_count);
    push_u32(&mut payload, 19);
    push_u32(&mut payload, 6);
    push_u32(&mut payload, 5);
    push_u32(&mut payload, 12);
    payload.extend_from_slice(&template_bytes);
    payload.extend_from_slice(&slot_bytes);
    payload.extend_from_slice(&text_bytes);
    payload.extend_from_slice(&hotspot_bytes);
    payload.extend_from_slice(&viewport_bytes);
    Ok(payload)
}

fn push_compact_ui_rect(
    bytes: &mut Vec<u8>,
    strings: &mut Vec<String>,
    string_refs: &mut HashMap<String, u32>,
    rect: &Value,
) {
    for key in [
        "id",
        "kind",
        "role",
        "label",
        "tooltip",
        "action",
        "itemId",
        "payloadKey",
    ] {
        push_u32(
            bytes,
            intern_compact_string(strings, string_refs, value_string(rect, key)),
        );
    }
    push_i32(bytes, value_i64(rect, "x").unwrap_or(0) as i32);
    push_i32(bytes, value_i64(rect, "y").unwrap_or(0) as i32);
    push_u32(bytes, value_u64(rect, "width").unwrap_or(0) as u32);
    push_u32(bytes, value_u64(rect, "height").unwrap_or(0) as u32);
}

pub fn build_compact_ui_binding_payload(
    bindings: &[Value],
    strings: &mut Vec<String>,
    string_refs: &mut HashMap<String, u32>,
) -> Result<Vec<u8>> {
    let mut row_bytes = Vec::new();
    for binding in bindings {
        for key in [
            "recipeId",
            "path",
            "payloadKey",
            "familyKey",
            "recipeType",
            "machineType",
            "templateKey",
            "templateSignature",
            "canonicalMachineFamily",
            "layoutKind",
        ] {
            push_u32(
                &mut row_bytes,
                intern_compact_string(strings, string_refs, value_string(binding, key)),
            );
        }
        let flags = if value_string(binding, "templateKey").is_some_and(|value| !value.is_empty()) {
            1
        } else {
            0
        };
        push_u32(&mut row_bytes, flags);
    }
    let mut payload = Vec::with_capacity(8 + 3 * 4 + row_bytes.len());
    payload.extend_from_slice(b"NEIUIB1\0");
    push_u32(&mut payload, 1);
    push_u32(&mut payload, bindings.len() as u32);
    push_u32(&mut payload, 11);
    payload.extend_from_slice(&row_bytes);
    Ok(payload)
}

pub fn build_compact_ui_string_payload(strings: &[String]) -> Result<Vec<u8>> {
    let mut string_offsets = Vec::<u32>::with_capacity(strings.len());
    let mut string_bytes = Vec::<u8>::new();
    for value in strings {
        string_offsets.push(string_bytes.len() as u32);
        string_bytes.extend_from_slice(value.as_bytes());
        string_bytes.push(0);
    }
    let mut payload = Vec::with_capacity(8 + 3 * 4 + string_offsets.len() * 4 + string_bytes.len());
    payload.extend_from_slice(b"NEIUIS1\0");
    push_u32(&mut payload, 1);
    push_u32(&mut payload, strings.len() as u32);
    push_u32(&mut payload, string_bytes.len() as u32);
    for offset in string_offsets {
        push_u32(&mut payload, offset);
    }
    payload.extend_from_slice(&string_bytes);
    Ok(payload)
}
