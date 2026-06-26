use crate::io::write_json_value;
use crate::json_ext::{json_array, read_json_file, value_string, value_u64};
use anyhow::Result;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub type CapturedUiFamilyKeyFn = fn(Option<&Value>, Option<&Value>) -> Option<String>;

pub fn compile_native_ui_layout_report(
    output: &Path,
    captured_ui_family_key: CapturedUiFamilyKeyFn,
) -> Result<Option<Value>> {
    let handler_layout_path = output.join("recipes").join("handler-layout-index.json");
    if !handler_layout_path.exists() {
        return Ok(None);
    }
    let ui_payload_path = output.join("recipes").join("ui-payload-index.json");
    let handler_layout_index = read_json_file(&handler_layout_path)?;
    let ui_payload_index = if ui_payload_path.exists() {
        read_json_file(&ui_payload_path)?
    } else {
        json!({ "recipes": [] })
    };
    let layouts = json_array(&handler_layout_index, "layouts");
    let recipes = json_array(&ui_payload_index, "recipes");
    let gt_layouts = layouts
        .iter()
        .filter(|layout| is_gregtech_native_layout(layout))
        .collect::<Vec<_>>();
    let layout_by_family = layouts
        .iter()
        .filter_map(|layout| Some((captured_ui_family_key(None, Some(layout))?, layout)))
        .collect::<BTreeMap<_, _>>();
    let gt_recipe_entries = recipes
        .iter()
        .filter(|entry| is_gregtech_recipe_ui_entry(entry))
        .collect::<Vec<_>>();
    let gt_recipe_layouts = gt_recipe_entries
        .iter()
        .filter_map(|entry| {
            value_string(entry, "familyKey")
                .and_then(|family_key| layout_by_family.get(&family_key).copied())
        })
        .collect::<Vec<_>>();

    let mut failures = Vec::new();
    let mut background_asset_gaps = Vec::new();
    if layouts.is_empty() {
        failures.push("handler layout index is empty".to_string());
    }
    if !gt_layouts.is_empty()
        && gt_layouts
            .iter()
            .filter(|layout| primitive_count(layout, "progressBars") > 0)
            .count()
            == 0
    {
        failures.push("gregtech-machine handler layouts have no drawable progressBars".to_string());
    }
    if !gt_recipe_layouts.is_empty()
        && gt_recipe_layouts
            .iter()
            .filter(|layout| primitive_count(layout, "progressBars") > 0)
            .count()
            == 0
    {
        failures
            .push("gregtech-machine recipe UI payloads have no drawable progressBars".to_string());
    }
    let gt_recipe_entries_with_background_regions = gt_recipe_layouts
        .iter()
        .filter(|layout| has_image_region(layout))
        .count();
    let gt_recipe_entries_with_native_backgrounds = gt_recipe_layouts
        .iter()
        .filter(|layout| has_native_background(layout))
        .count();
    if !gt_recipe_layouts.is_empty() && gt_recipe_entries_with_native_backgrounds == 0 {
        background_asset_gaps.push(
            "gregtech-machine recipe UI payloads have no nativeBackground capture facts"
                .to_string(),
        );
    }
    let geometry_status = if failures.is_empty() {
        "ready"
    } else {
        "blocked"
    };
    let background_status = if gt_recipe_entries_with_background_regions > 0 {
        "captured"
    } else if gt_recipe_entries_with_native_backgrounds > 0 {
        "semantic"
    } else if background_asset_gaps.is_empty() {
        "ready"
    } else {
        "missing"
    };

    let report = json!({
        "schemaVersion": "neonei/native-ui-layout-report/current",
        "generatedAt": "deterministic-rust-compiler",
        "status": geometry_status,
        "geometryStatus": geometry_status,
        "backgroundStatus": background_status,
        "source": {
            "handlerLayoutIndex": "recipes/handler-layout-index.json",
            "recipeUiPayloadIndex": if ui_payload_path.exists() { "recipes/ui-payload-index.json" } else { "" },
        },
        "counts": {
            "handlerLayouts": layouts.len(),
            "handlerLayoutsWithBackgroundRegions": layouts.iter().filter(|layout| has_image_region(layout)).count(),
            "handlerLayoutsWithProgressBars": layouts.iter().filter(|layout| primitive_count(layout, "progressBars") > 0).count(),
            "handlerLayoutsWithFluidBars": layouts.iter().filter(|layout| primitive_count(layout, "fluidBars") > 0).count(),
            "handlerLayoutsWithEnergyBars": layouts.iter().filter(|layout| primitive_count(layout, "energyBars") > 0).count(),
            "handlerLayoutsWithHotspots": layouts.iter().filter(|layout| primitive_count(layout, "hotspots") > 0).count(),
            "handlerLayoutsWithViewports": layouts.iter().filter(|layout| primitive_count(layout, "viewports") > 0).count(),
            "gregtechHandlerLayouts": gt_layouts.len(),
            "gregtechHandlerLayoutsWithBackgroundRegions": gt_layouts.iter().filter(|layout| has_image_region(layout)).count(),
            "gregtechHandlerLayoutsWithProgressBars": gt_layouts.iter().filter(|layout| primitive_count(layout, "progressBars") > 0).count(),
            "recipeUiPayloads": recipes.len(),
            "gregtechRecipeUiPayloads": gt_recipe_entries.len(),
            "gregtechRecipeUiPayloadsWithBackgroundRegions": gt_recipe_entries_with_background_regions,
            "gregtechRecipeUiPayloadsWithNativeBackgrounds": gt_recipe_entries_with_native_backgrounds,
            "gregtechRecipeUiPayloadsWithProgressBars": gt_recipe_layouts.iter().filter(|layout| primitive_count(layout, "progressBars") > 0).count(),
            "gregtechRecipeUiPayloadsWithHotspots": gt_recipe_layouts.iter().filter(|layout| primitive_count(layout, "hotspots") > 0).count(),
            "gregtechRecipeUiPayloadsWithViewports": gt_recipe_layouts.iter().filter(|layout| primitive_count(layout, "viewports") > 0).count(),
        },
        "samples": {
            "gregtechHandlerLayoutsMissingProgressBars": gt_layouts.iter()
                .filter(|layout| primitive_count(layout, "progressBars") == 0)
                .take(25)
                .map(|layout| json!({
                    "handlerKey": value_string(layout, "handlerKey"),
                    "handlerClass": value_string(layout, "handlerClass"),
                    "layoutKind": value_string(layout, "layoutKind"),
                }))
                .collect::<Vec<_>>(),
            "gregtechRecipePayloadsMissingProgressBars": gt_recipe_entries.iter()
                .filter(|entry| value_string(entry, "familyKey")
                    .and_then(|family_key| layout_by_family.get(&family_key).copied())
                    .map(|layout| primitive_count(layout, "progressBars") == 0)
                    .unwrap_or(true))
                .take(25)
                .map(|entry| json!({
                    "recipeId": value_string(entry, "recipeId"),
                    "familyKey": value_string(entry, "familyKey"),
                    "handlerKey": value_string(entry, "handlerKey"),
                }))
                .collect::<Vec<_>>(),
        },
        "failures": failures,
        "backgroundAssetGaps": background_asset_gaps,
    });
    let rust_dir = output.join("rust");
    fs::create_dir_all(&rust_dir)?;
    write_json_value(&rust_dir.join("native-ui-layout-report.json"), &report)?;
    Ok(Some(report))
}

fn is_gregtech_native_layout(layout: &Value) -> bool {
    value_string(layout, "canonicalMachineFamily")
        .map(|value| value.trim().eq_ignore_ascii_case("gregtech-machine"))
        .unwrap_or(false)
}

fn is_gregtech_recipe_ui_entry(entry: &Value) -> bool {
    entry
        .get("nativeLayout")
        .is_some_and(is_gregtech_native_layout)
        || value_string(entry, "familyKey")
            .map(|value| value.starts_with("gregtech-machine|"))
            .unwrap_or(false)
}

fn primitive_count(layout: &Value, key: &str) -> usize {
    layout
        .get(key)
        .and_then(Value::as_array)
        .map(|items| items.iter().filter(|item| has_drawable_rect(item)).count())
        .unwrap_or(0)
}

fn has_drawable_rect(value: &Value) -> bool {
    let width = value_u64(value, "width").unwrap_or(0);
    let height = value_u64(value, "height").unwrap_or(0);
    width > 0 && height > 0
}

fn has_image_region(value: &Value) -> bool {
    value.get("imageRegion").is_some_and(has_drawable_rect)
}

fn has_native_background(value: &Value) -> bool {
    let Some(background) = value.get("nativeBackground") else {
        return has_image_region(value);
    };
    let status = value_string(background, "status").unwrap_or_default();
    let kind = value_string(background, "kind").unwrap_or_default();
    matches!(status.as_str(), "captured" | "semantic")
        && matches!(kind.as_str(), "texture-region" | "gt-modular-ui")
}
