use crate::binary::{intern_compact_string, push_u32, write_binary_pack_payload};
use crate::io::write_json_value;
use crate::json_ext::{first_non_empty, nested_value_string, value_string, value_u64};
use crate::manifest::{
    read_json_collection, read_jsonl_file_values, read_jsonl_values, read_manifest,
    read_manifest_json, runtime_file_descriptors,
};
use crate::recipe_domain::{
    build_recipe_fragmentation_report, build_recipe_handler_metadata_report,
    captured_ui_family_key, classify_recipe_family_key, collect_recipe_item_ids,
    compact_fact_object, public_recipe_handler, public_recipe_layout, recipe_category_display_name,
    recipe_category_id_from_display_name, recipe_category_raw_id, recipe_id, recipe_machine_icon,
    RecipeCategoryAccumulator, RecipeHandlerContext,
};
use crate::recipe_ui_payload::{rust_recipe_ui_payload_relative_path, RecipeUiPayloadShardWriters};
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

pub fn compile_recipe_pack(
    input: &Path,
    output: &Path,
    strict: bool,
    debug_json: bool,
) -> Result<()> {
    let manifest = read_manifest(input)?;
    if !manifest.files.contains_key("recipeIndex") {
        return compile_dist_recipe_pack(input, output, strict, debug_json);
    }
    let recipe_index = read_manifest_json(input, &manifest, "recipeIndex")?
        .ok_or_else(|| anyhow!("recipe compiler blocked: recipeIndex is missing"))?;
    let handlers = read_jsonl_values(input, &manifest, "neiHandlers")?;
    let layouts = read_json_collection(
        input,
        &manifest,
        &["neiHandlerLayouts", "recipeLayouts"],
        Some("handler-layouts"),
    )?;
    let mut recipes = Vec::new();

    for shard in recipe_index
        .get("shards")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
    {
        let Some(path) = shard.get("path").and_then(Value::as_str) else {
            continue;
        };
        let shard_path = input.join(path.replace('\\', "/").trim_start_matches('/'));
        recipes.extend(read_jsonl_file_values(&shard_path)?);
    }

    if strict {
        let expected = recipe_index
            .get("recipeCount")
            .and_then(Value::as_u64)
            .unwrap_or(recipes.len() as u64);
        if expected != recipes.len() as u64 {
            return Err(anyhow!(
                "recipe compiler blocked: recipe count mismatch {} != {}",
                recipes.len(),
                expected
            ));
        }
    }

    let handler_context = RecipeHandlerContext::new(&handlers, &layouts);
    let handler_count = handlers.len();
    let public_handlers = handlers
        .iter()
        .map(public_recipe_handler)
        .collect::<Vec<_>>();
    let public_layouts = layouts.iter().map(public_recipe_layout).collect::<Vec<_>>();
    let handler_pack = if debug_json {
        public_handlers.clone()
    } else {
        Vec::new()
    };

    let mut produced_by: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    let mut used_in: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    let mut recipe_pack = Vec::new();
    let mut ui_payload_index = Vec::new();
    let mut ui_payload_shards = RecipeUiPayloadShardWriters::new(output)?;
    let mut category_map: BTreeMap<String, RecipeCategoryAccumulator> = BTreeMap::new();
    let mut recipe_count = 0usize;

    for recipe in &recipes {
        recipe_count += 1;
        let recipe_id = recipe_id(recipe);
        let (handler, layout) = handler_context.resolve(recipe);
        let public_handler = handler.map(public_recipe_handler);
        let public_layout = layout.map(public_recipe_layout);
        let raw_family_key = first_non_empty(&[
            value_string(recipe, "family"),
            value_string(recipe, "sourcePlugin"),
            value_string(recipe, "recipeType"),
            nested_value_string(recipe, &["machine", "machineId"]),
        ])
        .unwrap_or_else(|| "unknown".to_string());
        let family_key = captured_ui_family_key(handler, layout)
            .unwrap_or_else(|| classify_recipe_family_key(recipe, &raw_family_key, handler));
        let recipe_type = first_non_empty(&[
            value_string(recipe, "recipeType"),
            nested_value_string(recipe, &["machine", "machineId"]),
            Some(family_key.clone()),
        ])
        .unwrap_or_else(|| family_key.clone());
        let machine_type = first_non_empty(&[
            public_handler
                .as_ref()
                .and_then(|handler| value_string(handler, "localizedName")),
            public_handler
                .as_ref()
                .and_then(|handler| value_string(handler, "displayName")),
            nested_value_string(recipe, &["machine", "displayName"]),
            value_string(recipe, "displayName"),
            nested_value_string(recipe, &["machine", "machineId"]),
            Some(recipe_type.clone()),
        ])
        .unwrap_or_else(|| recipe_type.clone());
        let handler_key = public_handler
            .as_ref()
            .and_then(|handler| value_string(handler, "handlerKey"));
        let input_item_ids = collect_recipe_item_ids(
            recipe,
            &[
                "inputs",
                "inputItems",
                "itemInputs",
                "ingredients",
                "catalysts",
                "input",
            ],
        );
        let output_item_ids = collect_recipe_item_ids(
            recipe,
            &[
                "outputs",
                "outputItems",
                "itemOutputs",
                "results",
                "result",
                "output",
            ],
        );
        let machine_icon = recipe_machine_icon(recipe, public_handler.as_ref());
        let machine_info = json!({
            "machineType": machine_type,
            "machineId": nested_value_string(recipe, &["machine", "machineId"]).unwrap_or_else(|| recipe_type.clone()),
            "category": nested_value_string(recipe, &["machine", "category"]).unwrap_or_else(|| family_key.clone()),
            "iconInfo": nested_value_string(recipe, &["machine", "iconInfoRaw"]).unwrap_or_default(),
            "canonicalMachineFamily": public_handler
                .as_ref()
                .and_then(|handler| handler.get("canonicalMachineFamily").cloned())
                .unwrap_or(Value::Null),
            "catalystItemName": public_handler
                .as_ref()
                .and_then(|handler| handler.get("catalystItemName").cloned())
                .unwrap_or(Value::Null),
            "preferredMachineItemName": public_handler
                .as_ref()
                .and_then(|handler| handler.get("preferredMachineItemName").cloned())
                .unwrap_or(Value::Null),
            "gtMultiblockPreferred": public_handler
                .as_ref()
                .and_then(|handler| handler.get("gtMultiblockPreferred"))
                .and_then(Value::as_bool)
                .unwrap_or(false),
            "machineIcon": machine_icon.clone().unwrap_or(Value::Null),
        });
        let payload_meta = json!({
            "recipeId": recipe_id,
            "path": rust_recipe_ui_payload_relative_path(&recipe_id),
            "payloadKey": recipe_id,
            "familyKey": family_key,
            "recipeType": recipe_type,
            "machineType": machine_type,
            "handlerKey": handler_key,
            "handler": public_handler,
            "machineInfo": machine_info,
            "machineIcon": machine_icon,
            "nativeLayout": public_layout,
            "inputItemIds": input_item_ids,
            "outputItemIds": output_item_ids,
            "slotCount": { "input": input_item_ids.len(), "output": output_item_ids.len() },
            "presentation": {
                "surface": nested_value_string(recipe, &["machine", "machineId"]).unwrap_or_else(|| recipe_type.clone()),
                "density": if input_item_ids.len() + output_item_ids.len() > 12 { "dense" } else { "normal" },
            },
        });
        let payload_index_entry = json!({
            "recipeId": recipe_id,
            "path": rust_recipe_ui_payload_relative_path(&recipe_id),
            "payloadKey": recipe_id,
            "familyKey": family_key,
            "recipeType": recipe_type,
            "machineType": machine_type,
            "handlerKey": handler_key,
            "nativeLayout": public_layout,
        });
        let mut payload_entry = payload_meta;
        if let Some(payload_object) = payload_entry.as_object_mut() {
            payload_object.insert(
                "schemaVersion".to_string(),
                Value::String("neonei/recipe-ui-payload/v1".to_string()),
            );
            if let Some(domain_facts) = compact_fact_object(recipe.get("domainFacts")) {
                payload_object.insert("domainFacts".to_string(), domain_facts);
            }
            if let Some(metadata_facts) = compact_fact_object(recipe.get("metadata")) {
                payload_object.insert("metadata".to_string(), metadata_facts);
            }
            if let Some(layout_facts) = compact_fact_object(recipe.get("layout")) {
                payload_object.insert("layout".to_string(), layout_facts);
            }
            payload_object.remove("path");
            payload_object.remove("payloadKey");
        }
        ui_payload_shards.write_payload(&recipe_id, &payload_entry)?;
        ui_payload_index.push(payload_index_entry);

        let category_display_name = recipe_category_display_name(recipe, handler);
        let raw_category_id = recipe_category_raw_id(recipe, handler);
        let category_id =
            recipe_category_id_from_display_name(&category_display_name, &raw_category_id);
        let category =
            category_map
                .entry(category_id.clone())
                .or_insert_with(|| RecipeCategoryAccumulator {
                    category_id,
                    recipe_count: 0,
                    display_name: category_display_name.clone(),
                    source_category_ids: Vec::new(),
                    handler: public_handler.clone(),
                    native_layout: public_layout.clone(),
                    machine_icon: machine_icon.clone(),
                });
        category.recipe_count += 1;
        if category.machine_icon.is_none() {
            category.machine_icon = machine_icon.clone();
        }
        if !category.source_category_ids.contains(&raw_category_id) {
            category.source_category_ids.push(raw_category_id.clone());
        }

        let ref_value = json!({
            "recipeId": recipe_id,
            "categoryId": recipe_category_id_from_display_name(&category_display_name, &raw_category_id),
            "displayName": category_display_name,
        });

        for item_id in input_item_ids {
            used_in.entry(item_id).or_default().push(ref_value.clone());
        }
        for item_id in output_item_ids {
            produced_by
                .entry(item_id)
                .or_default()
                .push(ref_value.clone());
        }
        if debug_json {
            recipe_pack.push(recipe.clone());
        }
    }

    let mut item_ids = produced_by
        .keys()
        .chain(used_in.keys())
        .cloned()
        .collect::<Vec<_>>();
    item_ids.sort();
    item_ids.dedup();
    let item_index = item_ids
        .into_iter()
        .map(|item_id| {
            json!({
                "itemId": item_id,
                "producedBy": produced_by.remove(&item_id).unwrap_or_default(),
                "usedIn": used_in.remove(&item_id).unwrap_or_default(),
            })
        })
        .collect::<Vec<_>>();

    let category_index = category_map
        .into_values()
        .map(|category| {
            json!({
                "categoryId": category.category_id,
                "recipeCount": category.recipe_count,
                "displayName": category.display_name,
                "sourceCategoryIds": category.source_category_ids,
                "handler": category.handler,
                "nativeLayout": category.native_layout,
                "machineIcon": category.machine_icon,
            })
        })
        .collect::<Vec<_>>();

    let rust_dir = output.join("rust");
    fs::create_dir_all(&rust_dir)?;
    write_json_value(
        &rust_dir.join("recipe-handler-metadata-report.json"),
        &build_recipe_handler_metadata_report(&public_handlers, &public_layouts, &category_index),
    )?;
    write_json_value(
        &rust_dir.join("recipe-fragmentation-report.json"),
        &build_recipe_fragmentation_report(&category_index),
    )?;
    let recipes_dir = output.join("recipes");
    fs::create_dir_all(&recipes_dir)?;
    write_json_value(
        &recipes_dir.join("handler-index.json"),
        &json!({
            "schemaVersion": "neonei/recipe-handler-index/v1",
            "handlers": public_handlers.clone(),
        }),
    )?;
    write_json_value(
        &recipes_dir.join("handler-layout-index.json"),
        &json!({
            "schemaVersion": "neonei/recipe-handler-layout-index/v1",
            "layouts": public_layouts.clone(),
        }),
    )?;
    write_json_value(
        &recipes_dir.join("recipe-category-index.json"),
        &json!({
            "schemaVersion": "neonei/recipe-category-index/v1",
            "categories": category_index.clone(),
        }),
    )?;
    write_json_value(
        &recipes_dir.join("item-index.json"),
        &json!({
            "schemaVersion": "neonei/recipe-item-index/v1",
            "items": item_index.clone(),
        }),
    )?;
    write_json_value(
        &recipes_dir.join("ui-payload-index.json"),
        &json!({
            "schemaVersion": "neonei/recipe-ui-payload-index/v1",
            "recipes": ui_payload_index.clone(),
        }),
    )?;
    ui_payload_shards.finish()?;
    let recipe_output_pack = json!({
        "schemaVersion": "neonei/rust-recipe-pack/current",
        "counts": {
            "recipes": recipe_count,
            "handlers": handler_count,
            "recipeItemIndexItems": item_index.len(),
            "uiPayloadIndexItems": ui_payload_index.len(),
            "categories": category_index.len(),
        },
        "recipes": recipe_pack,
        "handlers": handler_pack,
        "itemIndex": item_index,
        "uiPayloadIndex": ui_payload_index,
        "categoryIndex": category_index,
    });
    if debug_json {
        write_json_value(&rust_dir.join("recipe-pack.json"), &recipe_output_pack)?;
    }
    let compact_recipe_payload = build_compact_recipe_payload_from_pack(&recipe_output_pack)?;
    write_binary_pack_payload(
        &rust_dir.join("recipes.bin"),
        "neonei/recipe-pack/current",
        &compact_recipe_payload,
    )?;
    Ok(())
}

pub fn compile_dist_recipe_pack(
    input: &Path,
    output: &Path,
    strict: bool,
    debug_json: bool,
) -> Result<()> {
    let manifest = read_manifest(input)?;
    let recipe_files = runtime_file_descriptors(
        input,
        &manifest,
        &[
            ("itemIndex", "recipeItemIndex"),
            ("handlers", "recipeHandlers"),
            ("handlerLayouts", "recipeHandlerLayouts"),
            ("categoryIndex", "recipeCategories"),
            ("uiPayloadIndex", "recipeUiPayloadIndex"),
        ],
    )?;
    if strict
        && !recipe_files.iter().any(|value| {
            value
                .get("logicalName")
                .and_then(Value::as_str)
                .is_some_and(|value| value == "itemIndex")
        })
    {
        return Err(anyhow!(
            "recipe compiler blocked: recipeItemIndex is missing"
        ));
    }

    let recipe_output_pack = json!({
        "schemaVersion": "neonei/rust-recipe-pack/current",
        "sourceKind": "dist-data",
        "counts": {
            "files": recipe_files.len(),
        },
        "files": recipe_files,
    });

    let rust_dir = output.join("rust");
    fs::create_dir_all(&rust_dir)?;
    if debug_json {
        write_json_value(&rust_dir.join("recipe-pack.json"), &recipe_output_pack)?;
    }
    let compact_recipe_payload = build_compact_recipe_payload_from_pack(&recipe_output_pack)?;
    write_binary_pack_payload(
        &rust_dir.join("recipes.bin"),
        "neonei/recipe-pack/current",
        &compact_recipe_payload,
    )?;
    Ok(())
}

pub fn build_compact_recipe_payload_from_pack(pack: &Value) -> Result<Vec<u8>> {
    let mut strings = vec![String::new()];
    let mut string_refs = HashMap::new();
    string_refs.insert(String::new(), 0u32);
    let mut item_rows = Vec::<[u32; 5]>::new();
    let mut ref_rows = Vec::<[u32; 3]>::new();
    let mut ui_rows = Vec::<[u32; 7]>::new();
    let mut category_rows = Vec::<[u32; 7]>::new();
    let mut category_sources = Vec::<u32>::new();

    for item in pack
        .get("itemIndex")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let produced_start = ref_rows.len() as u32;
        for recipe_ref in item
            .get("producedBy")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            ref_rows.push([
                intern_compact_string(
                    &mut strings,
                    &mut string_refs,
                    value_string(recipe_ref, "recipeId"),
                ),
                intern_compact_string(
                    &mut strings,
                    &mut string_refs,
                    value_string(recipe_ref, "categoryId"),
                ),
                intern_compact_string(
                    &mut strings,
                    &mut string_refs,
                    value_string(recipe_ref, "displayName"),
                ),
            ]);
        }
        let produced_count = (ref_rows.len() as u32).saturating_sub(produced_start);
        let used_start = ref_rows.len() as u32;
        for recipe_ref in item
            .get("usedIn")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            ref_rows.push([
                intern_compact_string(
                    &mut strings,
                    &mut string_refs,
                    value_string(recipe_ref, "recipeId"),
                ),
                intern_compact_string(
                    &mut strings,
                    &mut string_refs,
                    value_string(recipe_ref, "categoryId"),
                ),
                intern_compact_string(
                    &mut strings,
                    &mut string_refs,
                    value_string(recipe_ref, "displayName"),
                ),
            ]);
        }
        let used_count = (ref_rows.len() as u32).saturating_sub(used_start);
        item_rows.push([
            intern_compact_string(&mut strings, &mut string_refs, value_string(item, "itemId")),
            produced_start,
            produced_count,
            used_start,
            used_count,
        ]);
    }

    for entry in pack
        .get("uiPayloadIndex")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        ui_rows.push([
            intern_compact_string(
                &mut strings,
                &mut string_refs,
                value_string(entry, "recipeId"),
            ),
            intern_compact_string(&mut strings, &mut string_refs, value_string(entry, "path")),
            intern_compact_string(
                &mut strings,
                &mut string_refs,
                value_string(entry, "payloadKey"),
            ),
            intern_compact_string(
                &mut strings,
                &mut string_refs,
                value_string(entry, "familyKey"),
            ),
            intern_compact_string(
                &mut strings,
                &mut string_refs,
                value_string(entry, "recipeType"),
            ),
            intern_compact_string(
                &mut strings,
                &mut string_refs,
                value_string(entry, "machineType"),
            ),
            intern_compact_string(
                &mut strings,
                &mut string_refs,
                value_string(entry, "handlerKey"),
            ),
        ]);
    }

    for category in pack
        .get("categoryIndex")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let source_start = category_sources.len() as u32;
        for source in category
            .get("sourceCategoryIds")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            category_sources.push(intern_compact_string(
                &mut strings,
                &mut string_refs,
                source.as_str().map(str::to_string),
            ));
        }
        let source_count = (category_sources.len() as u32).saturating_sub(source_start);
        category_rows.push([
            intern_compact_string(
                &mut strings,
                &mut string_refs,
                value_string(category, "categoryId"),
            ),
            intern_compact_string(
                &mut strings,
                &mut string_refs,
                value_string(category, "displayName"),
            ),
            value_u64(category, "recipeCount")
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32,
            source_start,
            source_count,
            intern_compact_string(
                &mut strings,
                &mut string_refs,
                nested_value_string(category, &["machineIcon", "itemId"]),
            ),
            intern_compact_string(
                &mut strings,
                &mut string_refs,
                nested_value_string(category, &["machineIcon", "renderAssetRef"]),
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

    let item_stride = 5u32;
    let ref_stride = 3u32;
    let ui_stride = 7u32;
    let category_stride = 7u32;
    let mut payload = Vec::with_capacity(
        8 + 11 * 4
            + string_offsets.len() * 4
            + item_rows.len() * item_stride as usize * 4
            + ref_rows.len() * ref_stride as usize * 4
            + ui_rows.len() * ui_stride as usize * 4
            + category_rows.len() * category_stride as usize * 4
            + category_sources.len() * 4
            + string_bytes.len(),
    );
    payload.extend_from_slice(b"NEIRCP1\0");
    push_u32(&mut payload, 1);
    push_u32(&mut payload, strings.len() as u32);
    push_u32(&mut payload, item_rows.len() as u32);
    push_u32(&mut payload, ref_rows.len() as u32);
    push_u32(&mut payload, ui_rows.len() as u32);
    push_u32(&mut payload, category_rows.len() as u32);
    push_u32(&mut payload, category_sources.len() as u32);
    push_u32(&mut payload, item_stride);
    push_u32(&mut payload, ref_stride);
    push_u32(&mut payload, ui_stride);
    push_u32(&mut payload, category_stride);
    for offset in string_offsets {
        push_u32(&mut payload, offset);
    }
    for row in item_rows {
        for value in row {
            push_u32(&mut payload, value);
        }
    }
    for row in ref_rows {
        for value in row {
            push_u32(&mut payload, value);
        }
    }
    for row in ui_rows {
        for value in row {
            push_u32(&mut payload, value);
        }
    }
    for row in category_rows {
        for value in row {
            push_u32(&mut payload, value);
        }
    }
    for source in category_sources {
        push_u32(&mut payload, source);
    }
    payload.extend_from_slice(&string_bytes);
    Ok(payload)
}
