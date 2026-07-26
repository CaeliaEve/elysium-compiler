use crate::json_ext::value_string;
use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;

const UI_PRESENTATION_CATALOG_JSON: &str = include_str!("../catalog/ui-presentation.json");

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UiPresentationCatalog {
    schema_version: String,
    entries: Vec<UiPresentationDescriptor>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UiPresentationDescriptor {
    capture_key: Option<String>,
    canonical_machine_family: Option<String>,
    family_key: String,
    presentation_surface: String,
    layout_id: String,
    renderer_id: String,
}

fn required_catalog_value(value: &str, label: &str) -> Result<String> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(anyhow!("ui presentation catalog {label} must not be empty"));
    }
    Ok(normalized.to_string())
}

fn load_catalog() -> Result<UiPresentationCatalog> {
    let catalog: UiPresentationCatalog = serde_json::from_str(UI_PRESENTATION_CATALOG_JSON)
        .context("parse built-in UI presentation catalog")?;
    if catalog.schema_version != "elysium-compiler/ui-presentation-catalog/v1" {
        return Err(anyhow!(
            "ui presentation catalog schema mismatch: {}",
            catalog.schema_version
        ));
    }
    Ok(catalog)
}

pub fn enrich_ui_templates_with_presentation(templates: Vec<Value>) -> Result<Vec<Value>> {
    let catalog = load_catalog()?;
    let mut by_capture_key = BTreeMap::<String, UiPresentationDescriptor>::new();
    let mut by_canonical_family = BTreeMap::<String, UiPresentationDescriptor>::new();
    for descriptor in catalog.entries {
        required_catalog_value(&descriptor.family_key, "familyKey")?;
        required_catalog_value(&descriptor.presentation_surface, "presentationSurface")?;
        required_catalog_value(&descriptor.layout_id, "layoutId")?;
        required_catalog_value(&descriptor.renderer_id, "rendererId")?;
        let capture_key = descriptor
            .capture_key
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);
        let canonical_family = descriptor
            .canonical_machine_family
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);
        match (capture_key, canonical_family) {
            (Some(capture_key), None) => {
                if by_capture_key
                    .insert(capture_key.clone(), descriptor)
                    .is_some()
                {
                    return Err(anyhow!(
                        "ui presentation catalog has duplicate captureKey: {capture_key}"
                    ));
                }
            }
            (None, Some(canonical_family)) => {
                if by_canonical_family
                    .insert(canonical_family.clone(), descriptor)
                    .is_some()
                {
                    return Err(anyhow!(
                        "ui presentation catalog has duplicate canonicalMachineFamily: {canonical_family}"
                    ));
                }
            }
            _ => {
                return Err(anyhow!(
                    "ui presentation catalog entries must declare exactly one exact selector"
                ));
            }
        }
    }

    let mut templates = templates
        .into_iter()
        .map(|mut template| {
            let template_key = value_string(&template, "templateKey").unwrap_or_default();
            let capture_key = value_string(&template, "captureKey")
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| {
                    anyhow!("UI template {template_key} missing required v2 captureKey")
                })?;
            let canonical_family =
                value_string(&template, "canonicalMachineFamily").unwrap_or_default();
            let descriptor = by_capture_key
                .get(&capture_key)
                .or_else(|| by_canonical_family.get(&canonical_family))
                .ok_or_else(|| {
                    anyhow!(
                        "ui presentation catalog has no exact mapping for templateKey={template_key}, captureKey={capture_key}, canonicalMachineFamily={canonical_family}"
                    )
                })?;
            let object = template
                .as_object_mut()
                .ok_or_else(|| anyhow!("UI template {template_key} must be an object"))?;
            object.insert("captureKey".to_string(), Value::String(capture_key));
            object.insert(
                "familyKey".to_string(),
                Value::String(descriptor.family_key.clone()),
            );
            object.insert(
                "presentationSurface".to_string(),
                Value::String(descriptor.presentation_surface.clone()),
            );
            object.insert(
                "layoutId".to_string(),
                Value::String(descriptor.layout_id.clone()),
            );
            object.insert(
                "rendererId".to_string(),
                Value::String(descriptor.renderer_id.clone()),
            );
            Ok(template)
        })
        .collect::<Result<Vec<_>>>()?;
    let mut capture_keys = templates
        .iter()
        .filter_map(|template| value_string(template, "captureKey"))
        .collect::<std::collections::BTreeSet<_>>();
    for (canonical_family, descriptor) in &by_canonical_family {
        if capture_keys.insert(canonical_family.clone()) {
            templates.push(synthetic_web_template(canonical_family, descriptor));
        }
    }
    templates.sort_by(|left, right| {
        value_string(left, "templateKey").cmp(&value_string(right, "templateKey"))
    });
    Ok(templates)
}

fn synthetic_web_template(canonical_family: &str, descriptor: &UiPresentationDescriptor) -> Value {
    const WIDTH: u64 = 166;
    const HEIGHT: u64 = 65;
    serde_json::json!({
        "templateKey": format!("web-authored/{canonical_family}"),
        "templateSignature": format!("catalog:{canonical_family}:{}", descriptor.renderer_id),
        "captureKey": canonical_family,
        "canonicalMachineFamily": canonical_family,
        "layoutKind": descriptor.layout_id,
        "coordinateSpace": "nei_pixels",
        "scaleMode": "uniform-scale",
        "anchor": "top-left",
        "width": WIDTH,
        "height": HEIGHT,
        "yShift": 0,
        "maxRecipesPerPage": 1,
        "imageResource": "",
        "nativeBackground": {
            "status": "semantic",
            "kind": "canonical-nei-template",
            "coordinateSpace": "nei_pixels",
            "scaleMode": "uniform-scale",
            "anchor": "top-left",
            "scaling": "nine-slice",
            "width": WIDTH,
            "height": HEIGHT,
            "texture": { "width": 64, "height": 64, "borderU": 2, "borderV": 2 },
            "recipeBackgroundOffset": { "x": 0, "y": 0 },
            "recipeBackgroundSize": { "width": WIDTH, "height": HEIGHT },
            "captureRequired": false
        },
        "handlerCount": 0,
        "slotCount": 0,
        "handlerIds": [],
        "handlerClasses": [],
        "modIds": [],
        "slots": [],
        "dynamicPrimitives": [],
        "textOverlays": [],
        "hotspots": [],
        "viewports": [],
        "familyKey": descriptor.family_key,
        "presentationSurface": descriptor.presentation_surface,
        "layoutId": descriptor.layout_id,
        "rendererId": descriptor.renderer_id
    })
}
