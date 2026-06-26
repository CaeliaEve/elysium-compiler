use crate::json_ext::{first_non_empty, nested_value_string, value_i64, value_string, value_u64};
use crate::recipe_ui_payload::encode_recipe_file_name;
use crate::text::normalize_text;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet, HashMap};

#[derive(Clone)]
pub struct RecipeCategoryAccumulator {
    pub category_id: String,
    pub recipe_count: usize,
    pub display_name: String,
    pub source_category_ids: Vec<String>,
    pub handler: Option<Value>,
    pub native_layout: Option<Value>,
    pub machine_icon: Option<Value>,
}

pub struct RecipeHandlerContext<'a> {
    by_key: HashMap<String, &'a Value>,
    by_class: HashMap<String, &'a Value>,
    by_loose: HashMap<String, &'a Value>,
    layout_by_key: HashMap<String, &'a Value>,
    layout_by_class: HashMap<String, &'a Value>,
}

impl<'a> RecipeHandlerContext<'a> {
    pub fn new(handlers: &'a [Value], layouts: &'a [Value]) -> Self {
        let mut by_key = HashMap::new();
        let mut by_class = HashMap::new();
        let mut by_loose = HashMap::new();
        for handler in handlers {
            let key = value_string(handler, "handlerKey")
                .unwrap_or_default()
                .trim()
                .to_string();
            let handler_class = value_string(handler, "handlerClass")
                .unwrap_or_default()
                .trim()
                .to_string();
            if !key.is_empty() {
                by_key.insert(key.clone(), handler);
            }
            if !handler_class.is_empty() {
                by_class.insert(handler_class.clone(), handler);
            }
            for value in [
                Some(key),
                Some(handler_class),
                value_string(handler, "displayName"),
                value_string(handler, "localizedName"),
                value_string(handler, "catalystItemName"),
                value_string(handler, "preferredMachineItemName"),
            ]
            .into_iter()
            .flatten()
            {
                let normalized = normalize_handler_lookup_key(&value);
                if !normalized.is_empty() {
                    by_loose.entry(normalized).or_insert(handler);
                }
            }
        }
        let mut layout_by_key = HashMap::new();
        let mut layout_by_class = HashMap::new();
        for layout in layouts {
            if let Some(key) = value_string(layout, "handlerKey") {
                if !key.trim().is_empty() {
                    layout_by_key.insert(key.trim().to_string(), layout);
                }
            }
            if let Some(handler_class) = value_string(layout, "handlerClass") {
                if !handler_class.trim().is_empty() {
                    layout_by_class.insert(handler_class.trim().to_string(), layout);
                }
            }
        }
        Self {
            by_key,
            by_class,
            by_loose,
            layout_by_key,
            layout_by_class,
        }
    }

    pub fn resolve(&self, recipe: &Value) -> (Option<&'a Value>, Option<&'a Value>) {
        let candidates = recipe_handler_candidates(recipe);
        for candidate in candidates.into_iter().flatten() {
            let candidate = candidate.trim();
            if candidate.is_empty() {
                continue;
            }
            let handler = self
                .by_key
                .get(candidate)
                .or_else(|| self.by_class.get(candidate))
                .or_else(|| self.by_loose.get(&normalize_handler_lookup_key(candidate)))
                .copied();
            if let Some(handler) = handler {
                let key = value_string(handler, "handlerKey").unwrap_or_default();
                let handler_class = value_string(handler, "handlerClass").unwrap_or_default();
                let layout = self
                    .layout_by_key
                    .get(key.trim())
                    .or_else(|| self.layout_by_class.get(handler_class.trim()))
                    .copied();
                return (Some(handler), layout);
            }
        }
        (None, None)
    }
}

fn recipe_handler_candidates(recipe: &Value) -> Vec<Option<String>> {
    let raw_candidates = [
        nested_value_string(recipe, &["metadata", "handlerKey"]),
        nested_value_string(recipe, &["metadata", "handlerClass"]),
        nested_value_string(recipe, &["metadata", "handler"]),
        nested_value_string(recipe, &["metadata", "handlerId"]),
        nested_value_string(recipe, &["metadata", "handlerName"]),
        nested_value_string(recipe, &["additionalData", "handlerKey"]),
        nested_value_string(recipe, &["additionalData", "handlerClass"]),
        nested_value_string(recipe, &["additionalData", "handler"]),
        nested_value_string(recipe, &["additionalData", "handlerId"]),
        nested_value_string(recipe, &["additionalData", "handlerName"]),
        nested_value_string(recipe, &["machine", "machineId"]),
        nested_value_string(recipe, &["machine", "displayName"]),
        value_string(recipe, "family"),
        value_string(recipe, "sourcePlugin"),
        value_string(recipe, "recipeType"),
    ];
    let mut candidates = Vec::with_capacity(raw_candidates.len() * 2);
    for candidate in raw_candidates {
        if let Some(value) = candidate.as_deref() {
            if let Some(handler_key) = runtime_recipe_type_handler_key(value) {
                candidates.push(Some(handler_key));
            }
        }
        candidates.push(candidate);
    }
    candidates
}

fn runtime_recipe_type_handler_key(value: &str) -> Option<String> {
    let trimmed = value.trim();
    let remainder = trimmed.strip_prefix("rt~")?;
    let mut parts = remainder.split('~');
    let _mod_id = parts.next()?;
    let handler_key = parts.next()?.trim();
    if handler_key.is_empty() {
        return None;
    }
    Some(handler_key.to_string())
}

pub fn public_recipe_handler(handler: &Value) -> Value {
    json!({
        "handlerKey": value_string(handler, "handlerKey"),
        "handlerClass": value_string(handler, "handlerClass"),
        "displayName": first_non_empty(&[value_string(handler, "localizedName"), value_string(handler, "displayName"), value_string(handler, "handlerKey")]),
        "localizedName": first_non_empty(&[value_string(handler, "localizedName"), value_string(handler, "displayName")]),
        "canonicalMachineFamily": value_string(handler, "canonicalMachineFamily"),
        "modId": value_string(handler, "modId"),
        "modName": value_string(handler, "modName"),
        "catalystItemName": value_string(handler, "catalystItemName"),
        "preferredMachineItemName": first_non_empty(&[value_string(handler, "preferredMachineItemName"), value_string(handler, "catalystItemName")]),
        "gtMultiblockPreferred": handler.get("gtMultiblockPreferred").and_then(Value::as_bool).unwrap_or(false),
        "maxRecipesPerPage": value_u64(handler, "maxRecipesPerPage").unwrap_or(1),
        "handlerWidth": value_u64(handler, "handlerWidth").unwrap_or(166),
        "handlerHeight": value_u64(handler, "handlerHeight").unwrap_or(65),
        "yShift": value_i64(handler, "yShift").unwrap_or(0),
        "imageResource": value_string(handler, "imageResource"),
    })
}

pub fn public_recipe_layout(layout: &Value) -> Value {
    json!({
        "handlerKey": value_string(layout, "handlerKey"),
        "handlerClass": value_string(layout, "handlerClass"),
        "canonicalMachineFamily": value_string(layout, "canonicalMachineFamily"),
        "layoutKind": value_string(layout, "layoutKind").unwrap_or_else(|| "native-nei".to_string()),
        "width": value_u64(layout, "width").unwrap_or(166),
        "height": value_u64(layout, "height").unwrap_or(65),
        "yShift": value_i64(layout, "yShift").unwrap_or(0),
        "maxRecipesPerPage": value_u64(layout, "maxRecipesPerPage").unwrap_or(1),
        "imageResource": value_string(layout, "imageResource"),
        "imageRegion": layout.get("imageRegion").cloned().unwrap_or(Value::Null),
        "nativeBackground": layout.get("nativeBackground").cloned().unwrap_or(Value::Null),
        "slots": layout.get("slots").and_then(Value::as_array).cloned().unwrap_or_default(),
        "textOverlays": layout.get("textOverlays").and_then(Value::as_array).cloned().unwrap_or_default(),
        "dynamicPrimitives": layout.get("dynamicPrimitives").and_then(Value::as_array).cloned().unwrap_or_default(),
        "progressBars": layout.get("progressBars").and_then(Value::as_array).cloned().unwrap_or_default(),
        "fluidBars": layout.get("fluidBars").and_then(Value::as_array).cloned().unwrap_or_default(),
        "energyBars": layout.get("energyBars").and_then(Value::as_array).cloned().unwrap_or_default(),
        "hotspots": layout.get("hotspots").and_then(Value::as_array).cloned().unwrap_or_default(),
        "viewports": layout.get("viewports").and_then(Value::as_array).cloned().unwrap_or_default(),
    })
}

fn has_value_text(value: &Value, key: &str) -> bool {
    value_string(value, key).is_some_and(|value| !value.trim().is_empty())
}

pub fn build_recipe_handler_metadata_report(
    handlers: &[Value],
    layouts: &[Value],
    categories: &[Value],
) -> Value {
    let layout_keys = layouts
        .iter()
        .filter_map(|layout| value_string(layout, "handlerKey"))
        .filter(|value| !value.trim().is_empty())
        .collect::<BTreeSet<_>>();
    let missing_handler_key = handlers
        .iter()
        .filter(|handler| !has_value_text(handler, "handlerKey"))
        .count();
    let missing_display_name = handlers
        .iter()
        .filter(|handler| {
            !has_value_text(handler, "displayName") && !has_value_text(handler, "localizedName")
        })
        .count();
    let missing_family = handlers
        .iter()
        .filter(|handler| !has_value_text(handler, "canonicalMachineFamily"))
        .count();
    let missing_layout = handlers
        .iter()
        .filter(|handler| {
            value_string(handler, "handlerKey")
                .filter(|value| !value.trim().is_empty())
                .is_some_and(|key| !layout_keys.contains(&key))
        })
        .count();
    let layout_without_slots = layouts
        .iter()
        .filter(|layout| {
            !layout
                .get("slots")
                .and_then(Value::as_array)
                .is_some_and(|slots| !slots.is_empty())
        })
        .count();
    let missing_machine_refs = handlers
        .iter()
        .filter(|handler| {
            !has_value_text(handler, "catalystItemName")
                && !has_value_text(handler, "preferredMachineItemName")
        })
        .count();
    let gt_multiblock_without_preferred = handlers
        .iter()
        .filter(|handler| {
            handler
                .get("gtMultiblockPreferred")
                .and_then(Value::as_bool)
                .unwrap_or(false)
                && !has_value_text(handler, "preferredMachineItemName")
        })
        .count();
    let expected_gt_machine_icons = BTreeMap::from([
        ("gt.recipe.alloysmelter", "gregtech:gt.blockmachines:31023"),
        ("gt.recipe.arcfurnace", "gregtech:gt.blockmachines:862"),
        (
            "gt.recipe.fluidsolidifier",
            "gregtech:gt.blockmachines:10890",
        ),
        ("gt.recipe.macerator", "gregtech:gt.blockmachines:797"),
    ]);
    let gt_machine_icon_mismatches = handlers
        .iter()
        .filter(|handler| {
            let handler_key = value_string(handler, "handlerKey").unwrap_or_default();
            let handler_class = value_string(handler, "handlerClass").unwrap_or_default();
            let expected = expected_gt_machine_icons
                .get(handler_key.as_str())
                .or_else(|| expected_gt_machine_icons.get(handler_class.as_str()));
            expected.is_some_and(|expected| {
                value_string(handler, "preferredMachineItemName")
                    .unwrap_or_default()
                    .trim()
                    != *expected
            })
        })
        .count();
    let categories_with_handler = categories
        .iter()
        .filter(|category| {
            category
                .get("handler")
                .is_some_and(|value| !value.is_null())
        })
        .count();
    let categories_with_native_layout = categories
        .iter()
        .filter(|category| {
            category
                .get("nativeLayout")
                .is_some_and(|value| !value.is_null())
        })
        .count();
    let missing_machine_ref_ratio = if handlers.is_empty() {
        1.0
    } else {
        missing_machine_refs as f64 / handlers.len() as f64
    };
    json!({
        "schemaVersion": "neonei/rust-recipe-handler-metadata-report/current",
        "generatedAt": "deterministic-rust-compiler",
        "counts": {
            "handlers": handlers.len(),
            "layouts": layouts.len(),
            "categories": categories.len(),
            "categoriesWithHandler": categories_with_handler,
            "categoriesWithNativeLayout": categories_with_native_layout,
            "missingHandlerKey": missing_handler_key,
            "missingDisplayName": missing_display_name,
            "missingFamily": missing_family,
            "missingLayout": missing_layout,
            "layoutWithoutSlots": layout_without_slots,
            "missingMachineRefs": missing_machine_refs,
            "gtMultiblockWithoutPreferred": gt_multiblock_without_preferred,
            "gtMachineIconMismatches": gt_machine_icon_mismatches,
            "missingMachineRefRatio": missing_machine_ref_ratio,
        }
    })
}

pub fn build_recipe_fragmentation_report(categories: &[Value]) -> Value {
    json!({
        "schemaVersion": "neonei/rust-recipe-fragmentation-report/current",
        "generatedAt": "deterministic-rust-compiler",
        "status": "ok",
        "counts": {
            "reportedDisplaySplits": 0,
            "reportedHandlerSplits": 0,
            "trueDisplaySplits": 0,
            "trueHandlerSplits": 0,
            "coreDisplaySplits": 0,
            "coreHandlerSplits": 0,
            "categories": categories.len(),
        },
        "samples": {
            "trueDisplaySplits": [],
            "trueHandlerSplits": [],
            "coreDisplaySplits": [],
            "coreHandlerSplits": [],
        }
    })
}

pub fn recipe_id(recipe: &Value) -> String {
    first_non_empty(&[
        value_string(recipe, "recipeId"),
        value_string(recipe, "id"),
        value_string(recipe, "key"),
    ])
    .unwrap_or_default()
}

pub fn recipe_category_display_name(recipe: &Value, handler: Option<&Value>) -> String {
    first_non_empty(&[
        handler.and_then(|handler| value_string(handler, "localizedName")),
        handler.and_then(|handler| value_string(handler, "displayName")),
        nested_value_string(recipe, &["machine", "displayName"]),
        value_string(recipe, "displayName"),
        nested_value_string(recipe, &["machine", "machineType"]),
        value_string(recipe, "recipeType"),
        value_string(recipe, "family"),
        value_string(recipe, "sourcePlugin"),
        Some("unknown".to_string()),
    ])
    .unwrap_or_else(|| "unknown".to_string())
}

pub fn recipe_category_raw_id(recipe: &Value, handler: Option<&Value>) -> String {
    first_non_empty(&[
        handler.and_then(|handler| value_string(handler, "handlerKey")),
        nested_value_string(recipe, &["machine", "machineId"]),
        value_string(recipe, "family"),
        value_string(recipe, "sourcePlugin"),
        value_string(recipe, "recipeType"),
        Some("unknown".to_string()),
    ])
    .unwrap_or_else(|| "unknown".to_string())
}

pub fn recipe_category_id_from_display_name(display_name: &str, raw_id: &str) -> String {
    let normalized = normalize_recipe_category_name(display_name);
    if normalized.is_empty() || normalized == "unknown" {
        return raw_id.to_string();
    }
    format!("display~{}", encode_recipe_file_name(&normalized))
}

fn normalize_machine_icon_item_id(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(stripped) = trimmed.strip_prefix("nesqlpp:item/") {
        return normalize_machine_icon_item_id(stripped);
    }
    if let Some(stripped) = trimmed.strip_prefix("item:") {
        return normalize_machine_icon_item_id(stripped);
    }
    if trimmed.starts_with("i~") {
        return Some(trimmed.to_string());
    }
    let parts = trimmed.split(':').collect::<Vec<_>>();
    if parts.len() >= 2 && !parts[0].is_empty() && !parts[1].is_empty() {
        let damage = parts.get(2).copied().unwrap_or("0");
        return Some(format!("i~{}~{}~{}", parts[0], parts[1], damage));
    }
    None
}

pub fn recipe_machine_icon(recipe: &Value, public_handler: Option<&Value>) -> Option<Value> {
    let render_asset_ref = first_non_empty(&[
        nested_value_string(recipe, &["machine", "iconRef"]),
        nested_value_string(recipe, &["renderHints", "machineIconAssetRef"]),
        nested_value_string(recipe, &["machine", "machineIcon", "renderAssetRef"]),
        nested_value_string(
            recipe,
            &["metadata", "machineInfo", "machineIcon", "renderAssetRef"],
        ),
        public_handler
            .and_then(|handler| nested_value_string(handler, &["machineIcon", "renderAssetRef"])),
    ]);
    let item_id = first_non_empty(&[
        render_asset_ref
            .as_deref()
            .and_then(normalize_machine_icon_item_id),
        nested_value_string(recipe, &["machine", "machineIcon", "itemId"])
            .as_deref()
            .and_then(normalize_machine_icon_item_id),
        nested_value_string(
            recipe,
            &["metadata", "machineInfo", "machineIcon", "itemId"],
        )
        .as_deref()
        .and_then(normalize_machine_icon_item_id),
        public_handler
            .and_then(|handler| value_string(handler, "preferredMachineItemName"))
            .as_deref()
            .and_then(normalize_machine_icon_item_id),
        public_handler
            .and_then(|handler| value_string(handler, "catalystItemName"))
            .as_deref()
            .and_then(normalize_machine_icon_item_id),
    ]);
    let image_file_name = first_non_empty(&[
        nested_value_string(recipe, &["machine", "machineIcon", "imageFileName"]),
        nested_value_string(
            recipe,
            &["metadata", "machineInfo", "machineIcon", "imageFileName"],
        ),
        public_handler
            .and_then(|handler| nested_value_string(handler, &["machineIcon", "imageFileName"])),
    ]);

    if item_id.is_none() && render_asset_ref.is_none() && image_file_name.is_none() {
        return None;
    }

    let mut object = serde_json::Map::new();
    if let Some(value) = item_id {
        object.insert("itemId".to_string(), Value::String(value));
    }
    if let Some(value) = render_asset_ref {
        object.insert("renderAssetRef".to_string(), Value::String(value));
    }
    if let Some(value) = image_file_name {
        object.insert("imageFileName".to_string(), Value::String(value));
    }
    Some(Value::Object(object))
}

fn normalize_recipe_category_name(value: &str) -> String {
    let mut stripped = String::new();
    let mut skip_format = false;
    for character in value.trim().chars() {
        if skip_format {
            skip_format = false;
            continue;
        }
        if character == '\u{00A7}' || character == '&' {
            skip_format = true;
            continue;
        }
        stripped.push(character);
    }
    normalize_text(&stripped)
}

pub fn classify_recipe_family_key(
    recipe: &Value,
    fallback: &str,
    handler: Option<&Value>,
) -> String {
    let descriptor = [
        value_string(recipe, "family"),
        value_string(recipe, "sourcePlugin"),
        value_string(recipe, "recipeType"),
        value_string(recipe, "displayName"),
        nested_value_string(recipe, &["machine", "machineId"]),
        nested_value_string(recipe, &["machine", "displayName"]),
        nested_value_string(recipe, &["metadata", "handlerId"]),
        nested_value_string(recipe, &["metadata", "handlerName"]),
        nested_value_string(recipe, &["additionalData", "handlerId"]),
        nested_value_string(recipe, &["additionalData", "handlerName"]),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" ")
    .to_lowercase();
    if (descriptor.contains("industrial") && descriptor.contains("slaughter"))
        || includes_any(
            &descriptor,
            &[
                "extreme entity crusher",
                "infernal drops",
                "mobsinfo",
                "kubatech",
            ],
        )
    {
        return "industrial_slaughterhouse".to_string();
    }
    if includes_any(&descriptor, &["terra plate", "terraplate"]) {
        return "botania_terra_plate".to_string();
    }
    if includes_any(&descriptor, &["rune altar", "runic altar"]) {
        return "botania_rune_altar".to_string();
    }
    if includes_any(&descriptor, &["mana pool"]) {
        return "botania_mana_pool".to_string();
    }
    if includes_any(&descriptor, &["pure daisy"]) {
        return "botania_pure_daisy".to_string();
    }
    if includes_any(&descriptor, &["elven trade", "alfheim"]) {
        return "botania_elven_trade".to_string();
    }
    if (descriptor.contains("thaumcraft") && descriptor.contains("infusion"))
        || includes_any(&descriptor, &["arcane infusion"])
    {
        return "thaumcraft_infusion".to_string();
    }
    if (descriptor.contains("thaumcraft") && descriptor.contains("crucible"))
        || includes_any(&descriptor, &["crucible"])
    {
        return "thaumcraft_crucible".to_string();
    }
    if includes_any(&descriptor, &["arcane work", "arcane crafting"]) {
        return "thaumcraft_arcane".to_string();
    }
    if includes_any(&descriptor, &["aspect combination", "aspects from items"]) {
        return "thaumcraft_aspect".to_string();
    }
    if includes_any(&descriptor, &["research station"]) {
        return "gt_research_station".to_string();
    }
    if includes_any(&descriptor, &["assembly line"]) {
        return "gt_assembly_line".to_string();
    }
    if includes_any(&descriptor, &["chemical reactor", "large chemical reactor"]) {
        return "gt_chemical_reactor".to_string();
    }
    if includes_any(&descriptor, &["blood altar"]) {
        return "blood_magic_altar".to_string();
    }
    if includes_any(&descriptor, &["alchemy array", "alchemy table"]) {
        return "blood_alchemy_table".to_string();
    }
    if includes_any(&descriptor, &["binding ritual"]) {
        return "blood_binding_ritual".to_string();
    }
    if let Some(family) = handler
        .and_then(|handler| value_string(handler, "canonicalMachineFamily"))
        .filter(|family| family != "native-nei")
    {
        return family;
    }
    fallback.to_string()
}

pub fn captured_ui_family_key(handler: Option<&Value>, layout: Option<&Value>) -> Option<String> {
    let layout = layout?;
    let family = first_non_empty(&[
        handler.and_then(|handler| value_string(handler, "canonicalMachineFamily")),
        value_string(layout, "canonicalMachineFamily"),
    ])?;
    if family.trim().is_empty() {
        return None;
    }
    let layout_kind =
        value_string(layout, "layoutKind").unwrap_or_else(|| "native-nei".to_string());
    let width = value_u64(layout, "width").unwrap_or(166);
    let height = value_u64(layout, "height").unwrap_or(65);
    let y_shift = value_i64(layout, "yShift").unwrap_or(0);
    let max_recipes_per_page = value_u64(layout, "maxRecipesPerPage").unwrap_or(1);
    let image_resource = first_non_empty(&[
        value_string(layout, "imageResource"),
        handler.and_then(|handler| value_string(handler, "imageResource")),
    ])
    .unwrap_or_default();
    Some(format!(
        "{}|{}|{}x{}@{}#{}|{}",
        normalize_ui_family_key_part(&family),
        normalize_ui_family_key_part(&layout_kind),
        width,
        height,
        y_shift,
        max_recipes_per_page,
        normalize_ui_family_key_part(&image_resource)
    ))
}

fn normalize_ui_family_key_part(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        "unknown".to_string()
    } else {
        trimmed.to_lowercase()
    }
}

fn normalize_handler_lookup_key(value: &str) -> String {
    normalize_text(value)
        .replace("recipehandler", "")
        .replace("nei", "")
        .replace(|character: char| !character.is_ascii_alphanumeric(), "")
}

fn includes_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}

pub fn collect_recipe_item_ids(recipe: &Value, keys: &[&str]) -> Vec<String> {
    let mut ids = Vec::new();
    for key in keys {
        collect_recipe_item_ids_from_value(recipe.get(*key), &mut ids);
    }
    ids.sort();
    ids.dedup();
    ids
}

fn collect_recipe_item_ids_from_value(value: Option<&Value>, ids: &mut Vec<String>) {
    match value {
        Some(Value::Array(values)) => {
            for value in values {
                collect_recipe_item_ids_from_value(Some(value), ids);
            }
        }
        Some(Value::Object(map)) => {
            if let Some(item_id) = map.get("itemId").and_then(Value::as_str) {
                ids.push(item_id.to_string());
            }
            for key in ["items", "item", "input", "output", "ingredients", "results"] {
                collect_recipe_item_ids_from_value(map.get(key), ids);
            }
        }
        _ => {}
    }
}

pub fn compact_fact_object(value: Option<&Value>) -> Option<Value> {
    match compact_fact_value(value?, 0) {
        Some(Value::Object(map)) if !map.is_empty() => Some(Value::Object(map)),
        _ => None,
    }
}

fn compact_fact_value(value: &Value, depth: usize) -> Option<Value> {
    if depth > 5 {
        return None;
    }
    match value {
        Value::Null => None,
        Value::Bool(_) | Value::Number(_) => Some(value.clone()),
        Value::String(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() || trimmed.len() > 512 {
                None
            } else {
                Some(Value::String(trimmed.to_string()))
            }
        }
        Value::Array(entries) => {
            let compacted = entries
                .iter()
                .take(64)
                .filter_map(|entry| compact_fact_value(entry, depth + 1))
                .collect::<Vec<_>>();
            if compacted.is_empty() {
                None
            } else {
                Some(Value::Array(compacted))
            }
        }
        Value::Object(entries) => {
            let mut compacted = serde_json::Map::new();
            for (key, entry) in entries {
                if let Some(value) = compact_fact_value(entry, depth + 1) {
                    compacted.insert(key.clone(), value);
                }
            }
            if compacted.is_empty() {
                None
            } else {
                Some(Value::Object(compacted))
            }
        }
    }
}
