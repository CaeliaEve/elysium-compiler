use crate::io::normalize_path;
use crate::json_ext::value_string;
use crate::manifest::portable_relative_path;
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

pub fn ui_template_catalog_templates(catalog: &Value) -> Vec<Value> {
    let mut templates = catalog
        .get("templates")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|template| {
            value_string(template, "templateKey").is_some_and(|value| !value.trim().is_empty())
        })
        .collect::<Vec<_>>();
    templates.sort_by(|left, right| {
        value_string(left, "templateKey").cmp(&value_string(right, "templateKey"))
    });
    templates
}

pub fn build_ui_template_bindings(recipe_index: &[Value], templates: &[Value]) -> Vec<Value> {
    let template_by_family = templates
        .iter()
        .filter_map(|template| Some((value_string(template, "familyKey")?, template)))
        .collect::<BTreeMap<_, _>>();
    let mut bindings = recipe_index
        .iter()
        .filter_map(|entry| {
            let recipe_id = value_string(entry, "recipeId")?;
            let family_key = value_string(entry, "familyKey").unwrap_or_default();
            let template = template_by_family.get(&family_key).copied();
            Some(json!({
                "recipeId": recipe_id,
                "path": value_string(entry, "path").unwrap_or_default(),
                "payloadKey": value_string(entry, "payloadKey").unwrap_or_default(),
                "familyKey": family_key,
                "recipeType": value_string(entry, "recipeType").unwrap_or_default(),
                "machineType": value_string(entry, "machineType").unwrap_or_default(),
                "templateKey": template.and_then(|template| value_string(template, "templateKey")).unwrap_or_default(),
                "templateSignature": template.and_then(|template| value_string(template, "templateSignature")).unwrap_or_default(),
                "canonicalMachineFamily": template.and_then(|template| value_string(template, "canonicalMachineFamily")).unwrap_or_default(),
                "layoutKind": template.and_then(|template| value_string(template, "layoutKind")).unwrap_or_default(),
            }))
        })
        .collect::<Vec<_>>();
    bindings.sort_by(|left, right| {
        value_string(left, "recipeId").cmp(&value_string(right, "recipeId"))
    });
    bindings
}

fn value_usize(value: &Value, key: &str) -> usize {
    value.get(key).and_then(Value::as_u64).unwrap_or(0) as usize
}

fn value_i64_field(value: &Value, key: &str) -> i64 {
    value.get(key).and_then(Value::as_i64).unwrap_or(0)
}

fn value_string_list(value: &Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::trim))
                .filter(|item| !item.is_empty())
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn template_handler_count(template: &Value) -> usize {
    value_usize(template, "handlerCount")
        .max(value_string_list(template, "handlerIds").len())
        .max(value_string_list(template, "handlerClasses").len())
}

fn template_mod_ids(template: &Value) -> Vec<String> {
    value_string_list(template, "modIds")
}

pub fn build_ui_template_catalog_report(source_resource: &str, templates: &[Value]) -> Value {
    let family_keys = templates
        .iter()
        .filter_map(|template| value_string(template, "familyKey"))
        .filter(|value| !value.is_empty())
        .collect::<BTreeSet<_>>();
    let layout_kinds = templates
        .iter()
        .filter_map(|template| value_string(template, "layoutKind"))
        .filter(|value| !value.is_empty())
        .collect::<BTreeSet<_>>();
    let handler_count = templates.iter().map(template_handler_count).sum::<usize>();
    let slot_count = templates.iter().map(ui_template_slot_count).sum::<usize>();
    let overlay_count = templates.iter().map(ui_template_text_count).sum::<usize>();

    json!({
        "schemaVersion": "neonei/ui-template-catalog/current",
        "generatedAt": "deterministic-rust-compiler",
        "source": {
            "kind": "compiled-dist-data",
            "resource": "rust/ui-pack/ui_template_catalog.json",
            "rawExportResource": source_resource,
            "censusSchemaVersion": "neonei/ui-family-census/current",
            "censusFamilyCount": family_keys.len(),
            "censusHandlerCount": handler_count,
            "layoutSpecProvider": "elysium-compiler",
        },
        "summary": {
            "handlerCount": handler_count,
            "templateCount": templates.len(),
            "familyCount": family_keys.len(),
            "layoutKindCount": layout_kinds.len(),
            "slotCount": slot_count,
            "overlayCount": overlay_count,
        },
        "templates": templates,
    })
}

pub fn build_ui_template_binding_index_report(bindings: &[Value], templates: &[Value]) -> Value {
    let family_keys = templates
        .iter()
        .filter_map(|template| value_string(template, "familyKey"))
        .filter(|value| !value.is_empty())
        .collect::<BTreeSet<_>>();
    let layout_kinds = templates
        .iter()
        .filter_map(|template| value_string(template, "layoutKind"))
        .filter(|value| !value.is_empty())
        .collect::<BTreeSet<_>>();
    let report_bindings = bindings
        .iter()
        .map(|entry| {
            let template_key = value_string(entry, "templateKey").unwrap_or_default();
            let template_signature = value_string(entry, "templateSignature").unwrap_or_default();
            let canonical_machine_family =
                value_string(entry, "canonicalMachineFamily").unwrap_or_default();
            let layout_kind = value_string(entry, "layoutKind").unwrap_or_default();
            json!({
                "recipeId": value_string(entry, "recipeId").unwrap_or_default(),
                "path": value_string(entry, "path").unwrap_or_default(),
                "payloadKey": value_string(entry, "payloadKey").unwrap_or_default(),
                "familyKey": value_string(entry, "familyKey").unwrap_or_default(),
                "recipeType": value_string(entry, "recipeType").unwrap_or_default(),
                "machineType": value_string(entry, "machineType").unwrap_or_default(),
                "templateKey": if template_key.is_empty() { Value::Null } else { Value::String(template_key) },
                "templateSignature": if template_signature.is_empty() { Value::Null } else { Value::String(template_signature) },
                "canonicalMachineFamily": if canonical_machine_family.is_empty() { Value::Null } else { Value::String(canonical_machine_family) },
                "layoutKind": if layout_kind.is_empty() { Value::Null } else { Value::String(layout_kind) },
            })
        })
        .collect::<Vec<_>>();
    let bound_recipe_count = report_bindings
        .iter()
        .filter(|entry| entry.get("templateKey").and_then(Value::as_str).is_some())
        .count();

    json!({
        "schemaVersion": "neonei/ui-template-binding-index/current",
        "generatedAt": "deterministic-rust-compiler",
        "source": {
            "recipeUiPayloadIndex": {
                "path": "recipes/ui-payload-index.json",
                "exists": true,
                "recipeCount": bindings.len(),
            },
            "uiTemplateCatalog": {
                "path": "rust/ui-pack/ui_template_catalog.json",
                "exists": true,
                "templateCount": templates.len(),
                "familyCount": family_keys.len(),
                "layoutKindCount": layout_kinds.len(),
            },
        },
        "summary": {
            "recipeCount": report_bindings.len(),
            "boundRecipeCount": bound_recipe_count,
            "unboundRecipeCount": report_bindings.len().saturating_sub(bound_recipe_count),
            "templateCount": templates.len(),
            "familyCount": family_keys.len(),
            "layoutKindCount": layout_kinds.len(),
        },
        "bindings": report_bindings,
    })
}

pub fn build_ui_family_census_report(templates: &[Value]) -> Value {
    let mut family_map = BTreeMap::<String, Value>::new();
    for template in templates {
        let family_key = value_string(template, "familyKey").unwrap_or_default();
        if family_key.is_empty() {
            continue;
        }
        let width = value_usize(template, "width");
        let height = value_usize(template, "height");
        let y_shift = value_i64_field(template, "yShift");
        let max_recipes_per_page = value_usize(template, "maxRecipesPerPage").max(1);
        let image_resource = value_string(template, "imageResource").unwrap_or_default();
        let handler_ids = value_string_list(template, "handlerIds");
        let handler_classes = value_string_list(template, "handlerClasses");
        let mod_ids = template_mod_ids(template);
        let member_count = template_handler_count(template).max(1);
        let mut members = Vec::with_capacity(member_count);
        for index in 0..member_count {
            let handler = handler_classes
                .get(index)
                .or_else(|| handler_ids.get(index))
                .cloned()
                .unwrap_or_else(|| format!("{}#{}", family_key, index));
            let mod_id = mod_ids
                .get(index)
                .or_else(|| mod_ids.first())
                .cloned()
                .unwrap_or_default();
            members.push(json!({
                "handler": handler,
                "modId": mod_id.clone(),
                "modName": mod_id.clone(),
                "itemName": value_string(template, "templateKey").unwrap_or_default(),
                "itemNotes": "",
                "modRequired": !mod_id.is_empty(),
                "excludedModId": "",
                "imageResource": image_resource.clone(),
                "handlerWidth": width,
                "handlerHeight": height,
                "yShift": y_shift,
                "maxRecipesPerPage": max_recipes_per_page,
            }));
        }
        if let Some(existing) = family_map.get_mut(&family_key) {
            if let Some(existing_members) =
                existing.get_mut("members").and_then(Value::as_array_mut)
            {
                existing_members.extend(members);
            }
            continue;
        }
        family_map.insert(
            family_key.clone(),
            json!({
                "familyKey": family_key,
                "canonicalMachineFamily": value_string(template, "canonicalMachineFamily").unwrap_or_default(),
                "layoutKind": value_string(template, "layoutKind").unwrap_or_default(),
                "width": width,
                "height": height,
                "yShift": y_shift,
                "maxRecipesPerPage": max_recipes_per_page,
                "imageResource": image_resource,
                "members": members,
            }),
        );
    }
    let families = family_map.into_values().collect::<Vec<_>>();
    let handler_count = families
        .iter()
        .map(|family| {
            family
                .get("members")
                .and_then(Value::as_array)
                .map(Vec::len)
                .unwrap_or(0)
        })
        .sum::<usize>();
    let mod_count = families
        .iter()
        .flat_map(|family| {
            family
                .get("members")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .filter_map(|member| value_string(member, "modId"))
        .filter(|value| !value.is_empty())
        .collect::<BTreeSet<_>>()
        .len();
    let layout_kind_count = families
        .iter()
        .filter_map(|family| value_string(family, "layoutKind"))
        .filter(|value| !value.is_empty())
        .collect::<BTreeSet<_>>()
        .len();
    let crafting_family_count = families
        .iter()
        .filter(|family| {
            value_string(family, "layoutKind")
                .unwrap_or_default()
                .contains("crafting")
        })
        .count();
    let machine_family_count = families
        .iter()
        .filter(|family| {
            value_string(family, "layoutKind")
                .unwrap_or_default()
                .contains("machine")
        })
        .count();

    json!({
        "schemaVersion": "neonei/ui-family-census/current",
        "generatedAt": "deterministic-rust-compiler",
        "source": {
            "kind": "compiled-ui-template-catalog",
            "resource": "rust/ui-pack/ui_template_catalog.json",
            "entryCount": handler_count,
            "classifier": "elysium-compiler",
        },
        "summary": {
            "handlerCount": handler_count,
            "familyCount": families.len(),
            "modCount": mod_count,
            "layoutKindCount": layout_kind_count,
            "nativeFamilyCount": families.len(),
            "craftingFamilyCount": crafting_family_count,
            "machineFamilyCount": machine_family_count,
        },
        "families": families,
    })
}

pub fn build_ui_assets_manifest(templates: &[Value]) -> Value {
    let mut assets_by_ref: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for template in templates {
        let asset = value_string(template, "imageResource").unwrap_or_default();
        if asset.trim().is_empty() {
            // Template backgrounds may be exported through the structured
            // nativeBackground contract instead of legacy imageResource.
        } else {
            let template_key = value_string(template, "templateKey").unwrap_or_default();
            assets_by_ref.entry(asset).or_default().push(template_key);
        }
        let template_key = value_string(template, "templateKey").unwrap_or_default();
        if let Some(asset) = template
            .get("nativeBackground")
            .and_then(|background| value_string(background, "assetRef"))
            .filter(|value| !value.trim().is_empty())
        {
            assets_by_ref.entry(asset).or_default().push(template_key);
        }
    }
    let assets = assets_by_ref
        .into_iter()
        .map(|(asset_ref, mut template_keys)| {
            template_keys.sort();
            template_keys.dedup();
            json!({
                "assetRef": asset_ref,
                "kind": "template-background",
                "templateKeys": template_keys,
            })
        })
        .collect::<Vec<_>>();
    json!({
        "schemaVersion": "neonei/ui-assets-manifest/current",
        "generatedAt": "deterministic-rust-compiler",
        "assets": assets,
    })
}

pub fn materialize_ui_background_assets(
    input: &Path,
    output: &Path,
    assets_manifest: &Value,
) -> Result<Value> {
    let mut copied = Vec::new();
    let mut missing = Vec::new();
    let Some(assets) = assets_manifest.get("assets").and_then(Value::as_array) else {
        return Ok(json!({
            "schemaVersion": "neonei/ui-background-assets/current",
            "copied": copied,
            "missing": missing,
        }));
    };
    for asset in assets {
        let asset_ref = value_string(asset, "assetRef").unwrap_or_default();
        if !asset_ref.starts_with("assets/ui-backgrounds/") {
            continue;
        }
        let Some(relative) = portable_relative_path(&asset_ref) else {
            missing.push(json!({
                "assetRef": asset_ref,
                "reason": "non-portable-path",
            }));
            continue;
        };
        let source = input.join(&relative);
        if !source.is_file() {
            missing.push(json!({
                "assetRef": asset_ref,
                "path": normalize_path(relative.as_path()),
                "reason": "raw-export-asset-missing",
            }));
            continue;
        }
        let target = output.join(&relative);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&source, &target).with_context(|| {
            format!(
                "copy UI background asset {} -> {}",
                source.display(),
                target.display()
            )
        })?;
        copied.push(json!({
            "assetRef": asset_ref,
            "path": normalize_path(relative.as_path()),
        }));
    }
    Ok(json!({
        "schemaVersion": "neonei/ui-background-assets/current",
        "copied": copied,
        "missing": missing,
    }))
}

pub fn ui_template_slot_count(template: &Value) -> usize {
    template
        .get("slots")
        .and_then(Value::as_array)
        .map(|values| values.len())
        .unwrap_or(0)
}

pub fn ui_template_text_count(template: &Value) -> usize {
    template
        .get("textOverlays")
        .and_then(Value::as_array)
        .map(|values| values.len())
        .unwrap_or(0)
}

pub fn ui_template_rect_count(template: &Value, key: &str) -> usize {
    template
        .get(key)
        .and_then(Value::as_array)
        .map(|values| values.len())
        .unwrap_or(0)
}

pub fn ui_template_rect_action_count(template: &Value, key: &str) -> usize {
    template
        .get(key)
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter(|rect| value_string(rect, "action").is_some_and(|value| !value.is_empty()))
                .count()
        })
        .unwrap_or(0)
}
