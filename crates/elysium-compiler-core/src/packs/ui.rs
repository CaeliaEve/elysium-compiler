use crate::binary::{intern_compact_string, push_i32, push_u32, write_binary_pack_payload};
use crate::io::write_json_value;
use crate::json_ext::{value_i64, value_string, value_u64};
use crate::manifest::{read_manifest, read_manifest_json};
use crate::native_ui_pack_abi::{
    ui_pack_format_report, NATIVE_UI_ANCHOR, NATIVE_UI_BACKGROUND_KINDS,
    NATIVE_UI_BACKGROUND_SCALING_NINE_SLICE, NATIVE_UI_COORDINATE_SPACE,
    NATIVE_UI_INTERACTION_KIND_ITEM_CLICK, NATIVE_UI_INTERACTION_KIND_NONE,
    NATIVE_UI_INTERACTION_PAYLOAD_SCHEMA, NATIVE_UI_INTERACTION_TARGET_ITEM,
    NATIVE_UI_INTERACTION_TARGET_NONE, NATIVE_UI_SCALE_MODE, UI_BINDING_MAGIC,
    UI_BINDING_PAYLOAD_VERSION, UI_BINDING_ROW_STRIDE_U32, UI_BINDING_SCHEMA,
    UI_PRIMITIVE_ROW_STRIDE_U32, UI_RECT_ROW_STRIDE_U32, UI_SLOT_ROW_STRIDE_U32, UI_STRING_MAGIC,
    UI_STRING_PAYLOAD_VERSION, UI_STRING_SCHEMA, UI_TEMPLATE_MAGIC, UI_TEMPLATE_PAYLOAD_VERSION,
    UI_TEMPLATE_ROW_STRIDE_U32, UI_TEMPLATE_SCHEMA, UI_TEXT_ROW_STRIDE_U32,
};
use crate::recipe_ui_payload::{
    build_raw_recipe_ui_payload_index, materialize_native_frame_assets,
    read_compiled_recipe_ui_payload_index,
};
use crate::ui_templates::{
    build_ui_assets_manifest, build_ui_family_census_report,
    build_ui_template_binding_index_report, build_ui_template_bindings,
    build_ui_template_catalog_report, materialize_ui_background_assets,
    ui_template_catalog_templates, ui_template_dynamic_primitive_count, ui_template_rect_count,
    ui_template_rect_interaction_count, ui_template_slot_count, ui_template_text_count,
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
    validate_ui_template_background_contracts(&templates)?;

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
    let native_frame_assets = materialize_native_frame_assets(input, output, &recipe_ui_index)?;
    let missing_ui_background_assets = ui_background_assets
        .get("missing")
        .and_then(Value::as_array)
        .map(|items| items.len())
        .unwrap_or(0);
    let missing_native_frame_assets = native_frame_assets
        .get("missing")
        .and_then(Value::as_array)
        .map(|items| items.len())
        .unwrap_or(0);
    if strict && missing_ui_background_assets > 0 {
        return Err(anyhow!(
            "ui-pack compiler blocked: {missing_ui_background_assets} native UI background asset(s) are missing"
        ));
    }
    if strict && missing_native_frame_assets > 0 {
        return Err(anyhow!(
            "ui-pack compiler blocked: {missing_native_frame_assets} native NEI frame asset(s) are missing"
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
        UI_TEMPLATE_SCHEMA,
        &template_payload,
    )?;
    write_binary_pack_payload(
        &ui_pack_dir.join("ui_bindings.bin"),
        UI_BINDING_SCHEMA,
        &binding_payload,
    )?;
    write_binary_pack_payload(
        &ui_pack_dir.join("ui_strings.bin"),
        UI_STRING_SCHEMA,
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
                "dynamicPrimitiveCount": templates.iter().map(ui_template_dynamic_primitive_count).sum::<usize>(),
                "hotspotCount": templates.iter().map(|template| ui_template_rect_count(template, "hotspots")).sum::<usize>(),
                "viewportCount": templates.iter().map(|template| ui_template_rect_count(template, "viewports")).sum::<usize>(),
                "hotspotInteractionCount": templates.iter().map(|template| ui_template_rect_interaction_count(template, "hotspots")).sum::<usize>(),
                "viewportInteractionCount": templates.iter().map(|template| ui_template_rect_interaction_count(template, "viewports")).sum::<usize>(),
                "stringCount": strings.len(),
                "assetCount": assets_manifest.get("assets").and_then(Value::as_array).map(|items| items.len()).unwrap_or(0),
            },
            "format": ui_pack_format_report(),
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
    let mut primitive_bytes = Vec::new();
    let mut hotspot_bytes = Vec::new();
    let mut viewport_bytes = Vec::new();
    let mut slot_count = 0u32;
    let mut text_count = 0u32;
    let mut primitive_count = 0u32;
    let mut hotspot_count = 0u32;
    let mut viewport_count = 0u32;

    for template in templates {
        let template_coordinate_space = required_template_contract_string(
            template,
            "coordinateSpace",
            NATIVE_UI_COORDINATE_SPACE,
        )?;
        let template_anchor =
            required_template_contract_string(template, "anchor", NATIVE_UI_ANCHOR)?;
        let template_scale_mode =
            required_template_contract_string(template, "scaleMode", NATIVE_UI_SCALE_MODE)?;
        let slot_start = slot_count;
        if let Some(slots) = template.get("slots").and_then(Value::as_array) {
            for slot in slots {
                push_u32(
                    &mut slot_bytes,
                    intern_compact_string(
                        strings,
                        string_refs,
                        Some(required_slot_string(slot, "role")?),
                    ),
                );
                push_u32(
                    &mut slot_bytes,
                    required_slot_u32(slot, "startIndex", SlotFieldPolicy::NonNegative)?,
                );
                push_u32(
                    &mut slot_bytes,
                    required_slot_u32(slot, "columns", SlotFieldPolicy::Positive)?,
                );
                push_u32(
                    &mut slot_bytes,
                    required_slot_u32(slot, "rows", SlotFieldPolicy::Positive)?,
                );
                push_i32(&mut slot_bytes, required_slot_i32(slot, "x")?);
                push_i32(&mut slot_bytes, required_slot_i32(slot, "y")?);
                push_u32(
                    &mut slot_bytes,
                    intern_compact_string(
                        strings,
                        string_refs,
                        Some(required_slot_contract_string(
                            slot,
                            "coordinateSpace",
                            NATIVE_UI_COORDINATE_SPACE,
                        )?),
                    ),
                );
                push_u32(
                    &mut slot_bytes,
                    intern_compact_string(
                        strings,
                        string_refs,
                        Some(required_slot_contract_string(
                            slot,
                            "anchor",
                            NATIVE_UI_ANCHOR,
                        )?),
                    ),
                );
                push_u32(
                    &mut slot_bytes,
                    required_slot_u32(slot, "slotWidth", SlotFieldPolicy::Positive)?,
                );
                push_u32(
                    &mut slot_bytes,
                    required_slot_u32(slot, "slotHeight", SlotFieldPolicy::Positive)?,
                );
                push_u32(
                    &mut slot_bytes,
                    required_slot_u32(slot, "pitchX", SlotFieldPolicy::Positive)?,
                );
                push_u32(
                    &mut slot_bytes,
                    required_slot_u32(slot, "pitchY", SlotFieldPolicy::Positive)?,
                );
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
                push_u32(
                    &mut text_bytes,
                    intern_compact_string(
                        strings,
                        string_refs,
                        Some(required_rect_contract_string(
                            overlay,
                            "coordinateSpace",
                            &template_coordinate_space,
                        )?),
                    ),
                );
                push_u32(
                    &mut text_bytes,
                    intern_compact_string(
                        strings,
                        string_refs,
                        Some(required_rect_contract_string(
                            overlay,
                            "anchor",
                            &template_anchor,
                        )?),
                    ),
                );
                text_count += 1;
            }
        }
        let text_total = text_count.saturating_sub(text_start);
        let primitive_start = primitive_count;
        let template_key = value_string(template, "templateKey").unwrap_or("<unknown>".to_string());
        ensure_no_legacy_dynamic_primitive_fields(template, &template_key)?;
        let primitives = required_dynamic_primitives(template, &template_key)?;
        for primitive in primitives {
            push_compact_ui_dynamic_primitive(
                &mut primitive_bytes,
                strings,
                string_refs,
                primitive,
                &template_coordinate_space,
                &template_anchor,
                &template_key,
            )?;
            primitive_count += 1;
        }
        let primitive_total = primitive_count.saturating_sub(primitive_start);
        let hotspot_start = hotspot_count;
        if let Some(hotspots) = template.get("hotspots").and_then(Value::as_array) {
            for hotspot in hotspots {
                push_compact_ui_rect(
                    &mut hotspot_bytes,
                    strings,
                    string_refs,
                    hotspot,
                    &template_coordinate_space,
                    &template_anchor,
                )?;
                hotspot_count += 1;
            }
        }
        let hotspot_total = hotspot_count.saturating_sub(hotspot_start);
        let viewport_start = viewport_count;
        if let Some(viewports) = template.get("viewports").and_then(Value::as_array) {
            for viewport in viewports {
                push_compact_ui_rect(
                    &mut viewport_bytes,
                    strings,
                    string_refs,
                    viewport,
                    &template_coordinate_space,
                    &template_anchor,
                )?;
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
        push_u32(
            &mut template_bytes,
            intern_compact_string(
                strings,
                string_refs,
                Some(template_coordinate_space.clone()),
            ),
        );
        push_u32(
            &mut template_bytes,
            intern_compact_string(strings, string_refs, Some(template_scale_mode.clone())),
        );
        push_u32(
            &mut template_bytes,
            intern_compact_string(strings, string_refs, Some(template_anchor.clone())),
        );
        let native_background_json = serde_json::to_string(required_object(
            template,
            "nativeBackground",
            &format!(
                "ui template {}",
                value_string(template, "templateKey").unwrap_or("<unknown>".to_string())
            ),
        )?)?;
        push_u32(
            &mut template_bytes,
            intern_compact_string(strings, string_refs, Some(native_background_json)),
        );
        push_u32(&mut template_bytes, primitive_start);
        push_u32(&mut template_bytes, primitive_total);
    }

    let mut payload = Vec::with_capacity(
        8 + 12 * 4
            + template_bytes.len()
            + slot_bytes.len()
            + text_bytes.len()
            + primitive_bytes.len()
            + hotspot_bytes.len()
            + viewport_bytes.len(),
    );
    payload.extend_from_slice(UI_TEMPLATE_MAGIC);
    push_u32(&mut payload, UI_TEMPLATE_PAYLOAD_VERSION);
    push_u32(&mut payload, templates.len() as u32);
    push_u32(&mut payload, slot_count);
    push_u32(&mut payload, text_count);
    push_u32(&mut payload, primitive_count);
    push_u32(&mut payload, hotspot_count);
    push_u32(&mut payload, viewport_count);
    push_u32(&mut payload, UI_TEMPLATE_ROW_STRIDE_U32);
    push_u32(&mut payload, UI_SLOT_ROW_STRIDE_U32);
    push_u32(&mut payload, UI_TEXT_ROW_STRIDE_U32);
    push_u32(&mut payload, UI_PRIMITIVE_ROW_STRIDE_U32);
    push_u32(&mut payload, UI_RECT_ROW_STRIDE_U32);
    payload.extend_from_slice(&template_bytes);
    payload.extend_from_slice(&slot_bytes);
    payload.extend_from_slice(&text_bytes);
    payload.extend_from_slice(&primitive_bytes);
    payload.extend_from_slice(&hotspot_bytes);
    payload.extend_from_slice(&viewport_bytes);
    Ok(payload)
}

enum SlotFieldPolicy {
    NonNegative,
    Positive,
}

fn ensure_no_legacy_dynamic_primitive_fields(template: &Value, template_key: &str) -> Result<()> {
    for legacy_key in ["progressBars", "fluidBars", "energyBars"] {
        if template.get(legacy_key).is_some() {
            return Err(anyhow!(
                "ui template {template_key} uses legacy dynamic primitive field forbidden by v9 ABI: {legacy_key}"
            ));
        }
    }
    Ok(())
}

fn required_dynamic_primitives<'a>(
    template: &'a Value,
    template_key: &str,
) -> Result<&'a Vec<Value>> {
    template
        .get("dynamicPrimitives")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            anyhow!("ui template {template_key} missing required v9 dynamicPrimitives array")
        })
}

fn validate_ui_template_background_contracts(templates: &[Value]) -> Result<()> {
    for template in templates {
        let template_key = value_string(template, "templateKey").unwrap_or("<unknown>".to_string());
        let label = format!("ui template background {template_key}");
        let coordinate_space = required_template_contract_string(
            template,
            "coordinateSpace",
            NATIVE_UI_COORDINATE_SPACE,
        )?;
        let scale_mode =
            required_template_contract_string(template, "scaleMode", NATIVE_UI_SCALE_MODE)?;
        let anchor = required_template_contract_string(template, "anchor", NATIVE_UI_ANCHOR)?;
        let surface_width = required_template_u32(template, "width", &label)?;
        let surface_height = required_template_u32(template, "height", &label)?;
        let background = required_object(template, "nativeBackground", &label)?;

        required_contract_string(
            background,
            "coordinateSpace",
            &coordinate_space,
            &format!("{label}.nativeBackground"),
        )?;
        required_contract_string(
            background,
            "scaleMode",
            &scale_mode,
            &format!("{label}.nativeBackground"),
        )?;
        required_contract_string(
            background,
            "anchor",
            &anchor,
            &format!("{label}.nativeBackground"),
        )?;
        let status = required_string(background, "status", &label)?;
        if status != "captured" && status != "semantic" {
            return Err(anyhow!(
                "{label}.nativeBackground status must be captured or semantic, got {status}"
            ));
        }
        let kind = required_string(background, "kind", &format!("{label}.nativeBackground"))?;
        if !NATIVE_UI_BACKGROUND_KINDS.contains(&kind.as_str()) {
            return Err(anyhow!(
                "{label}.nativeBackground unsupported kind: {}",
                kind
            ));
        }
        required_contract_string(
            background,
            "scaling",
            NATIVE_UI_BACKGROUND_SCALING_NINE_SLICE,
            &format!("{label}.nativeBackground"),
        )?;
        if status == "captured" {
            required_string(background, "assetRef", &format!("{label}.nativeBackground"))?;
        }
        let background_width =
            required_u32(background, "width", &format!("{label}.nativeBackground"))?;
        let background_height =
            required_u32(background, "height", &format!("{label}.nativeBackground"))?;
        if background_width != surface_width || background_height != surface_height {
            return Err(anyhow!(
                "{label}.nativeBackground surface mismatch: background={}x{}, template={}x{}",
                background_width,
                background_height,
                surface_width,
                surface_height
            ));
        }

        let texture = required_object(background, "texture", &format!("{label}.nativeBackground"))?;
        required_u32(
            texture,
            "width",
            &format!("{label}.nativeBackground.texture"),
        )?;
        required_u32(
            texture,
            "height",
            &format!("{label}.nativeBackground.texture"),
        )?;
        required_u32(
            texture,
            "borderU",
            &format!("{label}.nativeBackground.texture"),
        )?;
        required_u32(
            texture,
            "borderV",
            &format!("{label}.nativeBackground.texture"),
        )?;

        let offset = required_object(
            background,
            "recipeBackgroundOffset",
            &format!("{label}.nativeBackground"),
        )?;
        let size = required_object(
            background,
            "recipeBackgroundSize",
            &format!("{label}.nativeBackground"),
        )?;
        let x = required_u32(
            offset,
            "x",
            &format!("{label}.nativeBackground.recipeBackgroundOffset"),
        )?;
        let y = required_u32(
            offset,
            "y",
            &format!("{label}.nativeBackground.recipeBackgroundOffset"),
        )?;
        let width = required_u32(
            size,
            "width",
            &format!("{label}.nativeBackground.recipeBackgroundSize"),
        )?;
        let height = required_u32(
            size,
            "height",
            &format!("{label}.nativeBackground.recipeBackgroundSize"),
        )?;
        if width == 0
            || height == 0
            || x > surface_width.saturating_sub(width)
            || y > surface_height.saturating_sub(height)
        {
            return Err(anyhow!(
                "{label}.nativeBackground target rect out of bounds: {},{} {}x{} surface={}x{}",
                x,
                y,
                width,
                height,
                surface_width,
                surface_height
            ));
        }
    }
    Ok(())
}

fn required_object<'a>(value: &'a Value, key: &str, label: &str) -> Result<&'a Value> {
    value
        .get(key)
        .filter(|value| value.is_object())
        .ok_or_else(|| anyhow!("{label} missing required object field: {key}"))
}

fn required_string(value: &Value, key: &str, label: &str) -> Result<String> {
    value_string(value, key)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow!("{label} missing required string field: {key}"))
}

fn required_contract_string(
    value: &Value,
    key: &str,
    expected: &str,
    label: &str,
) -> Result<String> {
    let value = required_string(value, key, label)?;
    if value != expected {
        return Err(anyhow!(
            "{label} field {key} must be {expected}, got {value}"
        ));
    }
    Ok(value)
}

fn required_template_u32(value: &Value, key: &str, label: &str) -> Result<u32> {
    let value = required_u32(value, key, label)?;
    if value == 0 {
        return Err(anyhow!("{label} field must be positive: {key}"));
    }
    Ok(value)
}

fn required_u32(value: &Value, key: &str, label: &str) -> Result<u32> {
    let value =
        value_u64(value, key).ok_or_else(|| anyhow!("{label} missing integer field: {key}"))?;
    if value > u32::MAX as u64 {
        return Err(anyhow!("{label} field exceeds u32: {key}={value}"));
    }
    Ok(value as u32)
}

fn required_slot_string(slot: &Value, key: &str) -> Result<String> {
    let value = value_string(slot, key)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow!("ui template slot missing required string field: {key}"))?;
    Ok(value)
}

fn required_slot_contract_string(slot: &Value, key: &str, expected: &str) -> Result<String> {
    let value = required_slot_string(slot, key)?;
    if value != expected {
        return Err(anyhow!(
            "ui template slot field {key} must be {expected}, got {value}"
        ));
    }
    Ok(value)
}

fn required_template_contract_string(
    template: &Value,
    key: &str,
    expected: &str,
) -> Result<String> {
    let value = value_string(template, key)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow!("ui template missing required geometry contract field: {key}"))?;
    if value != expected {
        return Err(anyhow!(
            "ui template geometry contract field {key} must be {expected}, got {value}"
        ));
    }
    Ok(value)
}

fn required_rect_contract_string(rect: &Value, key: &str, expected: &str) -> Result<String> {
    let value = value_string(rect, key)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            anyhow!("ui template rect missing required geometry contract field: {key}")
        })?;
    if value != expected {
        return Err(anyhow!(
            "ui template rect field {key} must be {expected}, got {value}"
        ));
    }
    Ok(value)
}

fn required_slot_u32(slot: &Value, key: &str, policy: SlotFieldPolicy) -> Result<u32> {
    let value = value_u64(slot, key)
        .ok_or_else(|| anyhow!("ui template slot missing required integer field: {key}"))?;
    match policy {
        SlotFieldPolicy::NonNegative => {}
        SlotFieldPolicy::Positive if value == 0 => {
            return Err(anyhow!("ui template slot field must be positive: {key}"));
        }
        SlotFieldPolicy::Positive => {}
    }
    if value > u32::MAX as u64 {
        return Err(anyhow!("ui template slot field exceeds u32: {key}={value}"));
    }
    Ok(value as u32)
}

fn required_slot_i32(slot: &Value, key: &str) -> Result<i32> {
    let value = value_i64(slot, key)
        .ok_or_else(|| anyhow!("ui template slot missing required signed integer field: {key}"))?;
    if value < i32::MIN as i64 || value > i32::MAX as i64 {
        return Err(anyhow!("ui template slot field exceeds i32: {key}={value}"));
    }
    Ok(value as i32)
}

fn required_primitive_string(primitive: &Value, key: &str, label: &str) -> Result<String> {
    value_string(primitive, key)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow!("{label} missing required string field: {key}"))
}

fn optional_primitive_string(primitive: &Value, key: &str) -> Option<String> {
    value_string(primitive, key)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn required_primitive_contract_string(
    primitive: &Value,
    key: &str,
    expected: &str,
    label: &str,
) -> Result<String> {
    let value = required_primitive_string(primitive, key, label)?;
    if value != expected {
        return Err(anyhow!(
            "{label} field {key} must be {expected}, got {value}"
        ));
    }
    Ok(value)
}

fn required_primitive_orientation(primitive: &Value, label: &str) -> Result<String> {
    let orientation = required_primitive_string(primitive, "orientation", label)?;
    if orientation != "horizontal" && orientation != "vertical" {
        return Err(anyhow!(
            "{label} orientation must be horizontal or vertical, got {orientation}"
        ));
    }
    Ok(orientation)
}

fn required_primitive_u32(primitive: &Value, key: &str, label: &str) -> Result<u32> {
    let value =
        value_u64(primitive, key).ok_or_else(|| anyhow!("{label} missing integer field: {key}"))?;
    if value == 0 {
        return Err(anyhow!("{label} field must be positive: {key}"));
    }
    if value > u32::MAX as u64 {
        return Err(anyhow!("{label} field exceeds u32: {key}={value}"));
    }
    Ok(value as u32)
}

fn required_primitive_i32(primitive: &Value, key: &str, label: &str) -> Result<i32> {
    let value = value_i64(primitive, key)
        .ok_or_else(|| anyhow!("{label} missing signed integer field: {key}"))?;
    if value < i32::MIN as i64 || value > i32::MAX as i64 {
        return Err(anyhow!("{label} field exceeds i32: {key}={value}"));
    }
    Ok(value as i32)
}

fn push_compact_ui_dynamic_primitive(
    bytes: &mut Vec<u8>,
    strings: &mut Vec<String>,
    string_refs: &mut HashMap<String, u32>,
    primitive: &Value,
    coordinate_space: &str,
    anchor: &str,
    template_key: &str,
) -> Result<()> {
    let label = format!("ui template {template_key} dynamic primitive");
    let kind = required_primitive_string(primitive, "kind", &label)?;
    let orientation = required_primitive_orientation(primitive, &label)?;
    let coordinate_space =
        required_primitive_contract_string(primitive, "coordinateSpace", coordinate_space, &label)?;
    let anchor = required_primitive_contract_string(primitive, "anchor", anchor, &label)?;
    push_u32(
        bytes,
        intern_compact_string(strings, string_refs, Some(kind)),
    );
    push_u32(
        bytes,
        intern_compact_string(
            strings,
            string_refs,
            optional_primitive_string(primitive, "role"),
        ),
    );
    push_i32(bytes, required_primitive_i32(primitive, "x", &label)?);
    push_i32(bytes, required_primitive_i32(primitive, "y", &label)?);
    push_u32(bytes, required_primitive_u32(primitive, "width", &label)?);
    push_u32(bytes, required_primitive_u32(primitive, "height", &label)?);
    for value in [
        Some(coordinate_space),
        Some(anchor),
        Some(orientation),
        optional_primitive_string(primitive, "source"),
        optional_primitive_string(primitive, "trackColor"),
        optional_primitive_string(primitive, "fillColor"),
        optional_primitive_string(primitive, "borderColor"),
    ] {
        push_u32(bytes, intern_compact_string(strings, string_refs, value));
    }
    Ok(())
}

struct UiRectInteractionContract {
    kind: String,
    target_kind: String,
    target_id: String,
    payload_schema: String,
}

fn resolve_rect_interaction_contract(rect: &Value) -> Result<UiRectInteractionContract> {
    for legacy_key in ["action", "itemId", "payloadKey"] {
        if rect.get(legacy_key).is_some() {
            return Err(anyhow!(
                "ui template rect uses legacy interaction field forbidden by v7 ABI: {legacy_key}"
            ));
        }
    }
    let kind = required_string(rect, "interactionKind", "ui template rect")?;
    let target_kind = required_string(rect, "interactionTargetKind", "ui template rect")?;
    let payload_schema = required_rect_contract_string(
        rect,
        "interactionPayloadSchema",
        NATIVE_UI_INTERACTION_PAYLOAD_SCHEMA,
    )?;
    let target_id = value_string(rect, "interactionTargetId")
        .unwrap_or_default()
        .trim()
        .to_string();

    match (kind.as_str(), target_kind.as_str()) {
        (NATIVE_UI_INTERACTION_KIND_NONE, NATIVE_UI_INTERACTION_TARGET_NONE) => {
            if !target_id.is_empty() {
                return Err(anyhow!(
                    "ui template rect interactionTargetId must be empty for non-interactive rects"
                ));
            }
        }
        (NATIVE_UI_INTERACTION_KIND_ITEM_CLICK, NATIVE_UI_INTERACTION_TARGET_ITEM) => {
            if target_id.is_empty() {
                return Err(anyhow!(
                    "ui template rect item-click interaction requires interactionTargetId"
                ));
            }
        }
        _ => {
            return Err(anyhow!(
                "ui template rect unsupported interaction contract: kind={}, targetKind={}",
                kind,
                target_kind
            ));
        }
    }

    Ok(UiRectInteractionContract {
        kind,
        target_kind,
        target_id,
        payload_schema,
    })
}

fn push_compact_ui_rect(
    bytes: &mut Vec<u8>,
    strings: &mut Vec<String>,
    string_refs: &mut HashMap<String, u32>,
    rect: &Value,
    coordinate_space: &str,
    anchor: &str,
) -> Result<()> {
    let interaction = resolve_rect_interaction_contract(rect)?;
    for key in ["id", "kind", "role", "label", "tooltip"] {
        push_u32(
            bytes,
            intern_compact_string(strings, string_refs, value_string(rect, key)),
        );
    }
    push_i32(bytes, value_i64(rect, "x").unwrap_or(0) as i32);
    push_i32(bytes, value_i64(rect, "y").unwrap_or(0) as i32);
    push_u32(bytes, value_u64(rect, "width").unwrap_or(0) as u32);
    push_u32(bytes, value_u64(rect, "height").unwrap_or(0) as u32);
    push_u32(
        bytes,
        intern_compact_string(
            strings,
            string_refs,
            Some(required_rect_contract_string(
                rect,
                "coordinateSpace",
                coordinate_space,
            )?),
        ),
    );
    push_u32(
        bytes,
        intern_compact_string(
            strings,
            string_refs,
            Some(required_rect_contract_string(rect, "anchor", anchor)?),
        ),
    );
    for value in [
        interaction.kind,
        interaction.target_kind,
        interaction.target_id,
        interaction.payload_schema,
    ] {
        push_u32(
            bytes,
            intern_compact_string(strings, string_refs, Some(value)),
        );
    }
    Ok(())
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
    payload.extend_from_slice(UI_BINDING_MAGIC);
    push_u32(&mut payload, UI_BINDING_PAYLOAD_VERSION);
    push_u32(&mut payload, bindings.len() as u32);
    push_u32(&mut payload, UI_BINDING_ROW_STRIDE_U32);
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
    payload.extend_from_slice(UI_STRING_MAGIC);
    push_u32(&mut payload, UI_STRING_PAYLOAD_VERSION);
    push_u32(&mut payload, strings.len() as u32);
    push_u32(&mut payload, string_bytes.len() as u32);
    for offset in string_offsets {
        push_u32(&mut payload, offset);
    }
    payload.extend_from_slice(&string_bytes);
    Ok(payload)
}
