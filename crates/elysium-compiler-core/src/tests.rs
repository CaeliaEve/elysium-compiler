use crate::atlas_repair::select_group_representative;
use crate::cli::{Cli, Command, CompileScope};
use crate::commands::run_command;
use crate::io::{normalize_path, write_json_value};
use crate::json_ext::{value_string, value_u64};
use crate::native_ui_report;
use crate::packs::browser::build_compact_group_payload_from_groups;
use crate::packs::recipe::build_compact_recipe_payload_from_pack;
use crate::packs::search::{
    build_compact_search_payload_from_items, build_compact_string_payload_from_items,
};
use crate::packs::texture::{
    build_compact_animation_payload_from_table, build_compact_atlas_meta_payload_from_atlas_items,
    build_compact_texture_payload_from_atlas_items, normalize_runtime_atlas_file_path,
    normalize_timeline,
};
use crate::packs::ui::{
    build_compact_ui_binding_payload, build_compact_ui_string_payload,
    build_compact_ui_template_payload,
};
use crate::raw_export;
use crate::recipe_domain::{captured_ui_family_key, public_recipe_layout, RecipeHandlerContext};
use crate::recipe_ui_payload::rust_recipe_ui_payload_relative_path;
use crate::reports;
use crate::runtime;
use crate::texture_animation::{
    expected_animated_item, expected_animation_reason, promote_animation_facts_to_animated_atlas,
};
use crate::ui_templates::{
    build_ui_assets_manifest, build_ui_template_bindings, materialize_ui_background_assets,
};
use crate::validation::{validate_atlas_bounds, validate_frame_bounds};
use serde_json::json;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

fn compiler_fixture_path(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name)
}

fn read_fixture_json(path: impl AsRef<Path>) -> serde_json::Value {
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

fn compile_fixture(fixture: &str, scope: CompileScope, strict: bool) -> tempfile::TempDir {
    let output = tempfile::tempdir().unwrap();
    let report = output.path().join("compiler-report.json");
    run_command(Cli {
        command: Command::Compile {
            input: compiler_fixture_path(fixture),
            output: output.path().to_path_buf(),
            report,
            scope,
            threads: Some(1),
            strict,
            debug_json: false,
        },
    })
    .unwrap();
    output
}

fn assert_expected_json_matches(fixture: &str, output: &Path, relative_path: &str) {
    let expected = compiler_fixture_path("expected")
        .join(fixture)
        .join(relative_path);
    let actual = output.join(relative_path);
    assert_eq!(
        read_fixture_json(actual),
        read_fixture_json(expected),
        "expected output mismatch for {fixture}:{relative_path}"
    );
}

#[test]
fn normalize_path_uses_forward_slashes() {
    assert!(normalize_path(Path::new("a/b")).contains('/'));
}

#[test]
fn empty_runtime_summary_without_output() {
    let summary = reports::summarize_runtime_output(None).unwrap();
    assert!(summary.counts.is_empty());
    assert!(summary.sizes.is_empty());
}

#[test]
fn compact_group_pack_uses_native_binary_payload() {
    let groups = vec![json!({
        "groupKey": "thaumcraft:wands",
        "groupLabel": "??",
        "groupSize": 2,
        "representativeItemId": "i~thaumcraft~wand~0",
        "memberItemIds": ["i~thaumcraft~wand~0", "i~thaumcraft~wand~1"]
    })];
    let payload = build_compact_group_payload_from_groups(&groups).unwrap();
    assert_eq!(&payload[0..8], b"NEIGRP1\0");
    assert_eq!(u32::from_le_bytes(payload[8..12].try_into().unwrap()), 1);
    assert_eq!(u32::from_le_bytes(payload[12..16].try_into().unwrap()), 1);
    assert_eq!(u32::from_le_bytes(payload[20..24].try_into().unwrap()), 2);
    assert_eq!(u32::from_le_bytes(payload[24..28].try_into().unwrap()), 6);
}

#[test]
fn compact_animation_pack_uses_native_binary_payload() {
    let animations = vec![json!({
        "itemId": "i~botania~manaResource~4",
        "atlasFile": "textures/atlas/animated-main.webp",
        "frameDurationMs": 50,
        "timeline": [{ "frameIndex": 0, "durationMs": 50 }, { "frameIndex": 1, "durationMs": 75 }]
    })];
    let payload = build_compact_animation_payload_from_table(&animations).unwrap();
    assert_eq!(&payload[0..8], b"NEIANM1\0");
    assert_eq!(u32::from_le_bytes(payload[8..12].try_into().unwrap()), 1);
    assert_eq!(u32::from_le_bytes(payload[12..16].try_into().unwrap()), 1);
    assert_eq!(u32::from_le_bytes(payload[20..24].try_into().unwrap()), 2);
    assert_eq!(u32::from_le_bytes(payload[24..28].try_into().unwrap()), 5);
}

#[test]
fn compact_texture_pack_uses_native_binary_payload() {
    let items = vec![json!({
        "itemId": "minecraft:iron_ingot",
        "staticAtlas": { "atlasFile": "textures/atlas/static-main.webp", "x": 1, "y": 2, "width": 16, "height": 16 },
        "animatedAtlas": {
            "atlasFile": "textures/atlas/animated-main.webp",
            "frameDurationMs": 50,
            "frames": [{ "x": 3, "y": 4, "width": 16, "height": 16 }],
            "timeline": [{ "frameIndex": 0, "durationMs": 50 }]
        }
    })];
    let payload = build_compact_texture_payload_from_atlas_items(&items).unwrap();
    assert_eq!(&payload[0..8], b"NEITEX1\0");
    assert_eq!(u32::from_le_bytes(payload[8..12].try_into().unwrap()), 1);
    assert_eq!(u32::from_le_bytes(payload[12..16].try_into().unwrap()), 1);
    assert_eq!(u32::from_le_bytes(payload[20..24].try_into().unwrap()), 1);
    assert_eq!(u32::from_le_bytes(payload[24..28].try_into().unwrap()), 10);
}

#[test]
fn group_representative_prefers_drawable_variant_over_tiny_native_sprite() {
    let mut atlas_by_item = BTreeMap::new();
    atlas_by_item.insert(
        "i~ae2fc~wireless_fluid_terminal~0".to_string(),
        json!({
            "itemId": "i~ae2fc~wireless_fluid_terminal~0",
            "resolutionMode": "native_sprite",
            "staticAtlas": { "atlasFile": "textures/atlas/static.png", "x": 0, "y": 0, "width": 16, "height": 16 }
        }),
    );
    atlas_by_item.insert(
        "i~ae2fc~wireless_fluid_terminal~0~charged".to_string(),
        json!({
            "itemId": "i~ae2fc~wireless_fluid_terminal~0~charged",
            "resolutionMode": "native_sprite",
            "staticAtlas": { "atlasFile": "textures/atlas/static.png", "x": 16, "y": 0, "width": 64, "height": 64 }
        }),
    );
    let selected = select_group_representative(
        Some("i~ae2fc~wireless_fluid_terminal~0".to_string()),
        &[
            "i~ae2fc~wireless_fluid_terminal~0".to_string(),
            "i~ae2fc~wireless_fluid_terminal~0~charged".to_string(),
        ],
        &atlas_by_item,
    );
    assert_eq!(
        selected.as_deref(),
        Some("i~ae2fc~wireless_fluid_terminal~0~charged")
    );
}

#[test]
fn runtime_atlas_paths_are_dist_data_relative() {
    assert_eq!(
        normalize_runtime_atlas_file_path(Some(
            "assets/textures/atlas-assets/atlases/item-native-static.png".to_string()
        ))
        .as_deref(),
        Some("textures/atlas-assets/atlases/item-native-static.png")
    );
    assert_eq!(
        normalize_runtime_atlas_file_path(Some(
            "textures/atlas-assets/atlases/item-native-static.png".to_string()
        ))
        .as_deref(),
        Some("textures/atlas-assets/atlases/item-native-static.png")
    );
}

#[test]
fn compact_atlas_meta_pack_summarizes_atlas_files() {
    let items = vec![
        json!({
            "itemId": "minecraft:iron_ingot",
            "staticAtlas": { "atlasFile": "textures/atlas/static-main.webp", "atlasWidth": 2048, "atlasHeight": 2048, "x": 1, "y": 2, "width": 16, "height": 16 },
        }),
        json!({
            "itemId": "i~AWWayofTime~lifeEssence~0",
            "animatedAtlas": {
                "atlasFile": "textures/atlas/animated-main.webp",
                "atlasWidth": { "value": "2048" },
                "atlasHeight": { "value": "4096" },
                "frames": [
                    { "x": { "value": "0" }, "y": { "value": "0" }, "width": { "value": "16" }, "height": { "value": "16" } },
                    { "x": { "value": "16" }, "y": { "value": "0" }, "width": { "value": "16" }, "height": { "value": "16" } }
                ]
            }
        }),
    ];
    let payload = build_compact_atlas_meta_payload_from_atlas_items(&items).unwrap();
    assert_eq!(&payload[0..8], b"NEIATM1\0");
    assert_eq!(u32::from_le_bytes(payload[8..12].try_into().unwrap()), 1);
    assert_eq!(u32::from_le_bytes(payload[12..16].try_into().unwrap()), 2);
    assert_eq!(u32::from_le_bytes(payload[20..24].try_into().unwrap()), 6);
}

#[test]
fn atlas_bounds_validation_blocks_out_of_bounds_static_rects() {
    let valid = json!({
        "atlasFile": "textures/atlas/static-main.webp",
        "atlasWidth": 64,
        "atlasHeight": 64,
        "x": 48,
        "y": 48,
        "width": 16,
        "height": 16,
    });
    let invalid = json!({
        "atlasFile": "textures/atlas/static-main.webp",
        "atlasWidth": 64,
        "atlasHeight": 64,
        "x": 60,
        "y": 48,
        "width": 16,
        "height": 16,
    });
    let zero_sized = json!({
        "atlasFile": "textures/atlas/static-main.webp",
        "atlasWidth": 64,
        "atlasHeight": 64,
        "x": 0,
        "y": 0,
        "width": 0,
        "height": 16,
    });
    let mut invalid_bounds = Vec::new();
    validate_atlas_bounds("valid-item", "static", Some(&valid), &mut invalid_bounds);
    assert!(invalid_bounds.is_empty());

    validate_atlas_bounds("bad-item", "static", Some(&invalid), &mut invalid_bounds);
    validate_atlas_bounds(
        "zero-item",
        "static",
        Some(&zero_sized),
        &mut invalid_bounds,
    );
    assert_eq!(
        invalid_bounds,
        vec![
            "bad-item:static:out-of-bounds".to_string(),
            "zero-item:static:out-of-bounds".to_string(),
        ]
    );
}

#[test]
fn atlas_bounds_validation_allows_animated_frame_only_atlas() {
    let frame_only = json!({
        "atlasFile": "textures/atlas/animated-main.webp",
        "atlasWidth": 16,
        "atlasHeight": 128,
        "frameCount": 8,
        "frames": [[0, 0, 0, 16, 16], [1, 0, 16, 16, 16]],
    });
    let animated_with_bad_rect = json!({
        "atlasFile": "textures/atlas/animated-main.webp",
        "atlasWidth": 16,
        "atlasHeight": 128,
        "x": 8,
        "y": 120,
        "width": 16,
        "height": 16,
        "frames": [[0, 0, 0, 16, 16]],
    });
    let mut invalid_bounds = Vec::new();
    validate_atlas_bounds(
        "frame-only-item",
        "animated",
        Some(&frame_only),
        &mut invalid_bounds,
    );
    assert!(invalid_bounds.is_empty());

    validate_atlas_bounds(
        "bad-animated-item",
        "animated",
        Some(&animated_with_bad_rect),
        &mut invalid_bounds,
    );
    assert_eq!(
        invalid_bounds,
        vec!["bad-animated-item:animated:out-of-bounds".to_string()]
    );
}

#[test]
fn missing_texture_report_classifies_expected_animated_static_items() {
    let animation = json!({
        "assetId": "avaritia-singularity",
        "frameCount": 4,
        "frameDurationMs": 50,
        "timeline": [{ "frameIndex": 0, "durationMs": 50 }]
    });
    let native_sprite = json!({
        "assetId": "avaritia-singularity",
        "spriteMetadataFile": "assets/minecraft/textures/items/singularity.png.mcmeta"
    });
    assert!(expected_animated_item(
        Some(&animation),
        Some(&native_sprite)
    ));
    assert_eq!(
        expected_animation_reason(Some(&animation), Some(&native_sprite)),
        "native sprite metadata exists"
    );
    assert!(!expected_animated_item(None, None));
}

#[test]
fn native_sprite_snapshot_animation_facts_promote_static_atlas() {
    let atlas = json!({
        "items": [{
            "itemId": "i~Railcraft~cart.redstone.flux~0",
            "assetId": "nesqlpp:item/i~Railcraft~cart.redstone.flux~0",
            "hasStaticAtlas": true,
            "hasAnimatedAtlas": false,
            "staticAtlas": {
                "atlasFile": "assets/textures/atlas-assets/atlases/item-native-static-011.png",
                "atlasWidth": 2048,
                "atlasHeight": 2048,
                "x": 512,
                "y": 128,
                "width": 64,
                "height": 64
            }
        }]
    });
    let animation = json!({
        "assetId": "nesqlpp:item/i~Railcraft~cart.redstone.flux~0",
        "mode": "native_sprite_snapshot",
        "animationMode": "none",
        "frameCount": 20,
        "frameDurationMs": 100,
        "timeline": [
            { "timelineIndex": 0.0, "frameIndex": 0.0, "durationMs": 100.0 },
            { "timelineIndex": 1.0, "frameIndex": 1.0, "durationMs": 100.0 }
        ]
    });
    let mut animations = BTreeMap::new();
    animations.insert(
        "nesqlpp:item/i~Railcraft~cart.redstone.flux~0".to_string(),
        animation,
    );
    let promoted = promote_animation_facts_to_animated_atlas(&atlas, &animations, &BTreeMap::new());
    let item = &promoted["items"][0];
    assert_eq!(item["hasAnimatedAtlas"], json!(true));
    assert_eq!(item["animatedAtlas"]["frameCount"], json!(20));
    assert_eq!(
        item["animatedAtlas"]["frames"].as_array().unwrap().len(),
        20
    );
    assert_eq!(
        item["animatedAtlas"]["timeline"][1],
        json!({ "frameIndex": 1, "durationMs": 100 })
    );
}

#[test]
fn wrapped_numeric_texture_frames_compile_without_invalid_bounds() {
    let animated_atlas = json!({
        "atlasFile": "assets/textures/atlas-assets/animated-atlases/item-native-animated.png",
        "atlasWidth": { "value": "2048" },
        "atlasHeight": { "value": "4096" },
        "frameDurationMs": { "value": "50" },
        "frameCount": { "value": "2" },
        "frames": [
            {
                "index": { "value": "0" },
                "x": { "value": "0" },
                "y": { "value": "0" },
                "width": { "value": "16" },
                "height": { "value": "16" }
            },
            {
                "index": { "value": 1 },
                "x": { "value": 16 },
                "y": { "value": 0 },
                "width": { "value": 16 },
                "height": { "value": 16 }
            }
        ],
        "timeline": [
            {
                "timelineIndex": { "value": "0" },
                "frameIndex": { "value": "0" },
                "durationMs": { "value": "50" }
            },
            {
                "timelineIndex": { "value": "1" },
                "frameIndex": { "value": "1" },
                "durationMs": { "value": "75" }
            }
        ]
    });

    let mut invalid_bounds = Vec::new();
    validate_frame_bounds(
        "i~AWWayofTime~lifeEssence~0",
        Some(&animated_atlas),
        &mut invalid_bounds,
    );
    assert!(invalid_bounds.is_empty(), "{invalid_bounds:?}");

    let normalized = normalize_timeline(
        Some(&animated_atlas),
        value_u64(&animated_atlas, "frameDurationMs"),
    );
    assert_eq!(
        normalized
            .as_array()
            .and_then(|values| values.get(1))
            .and_then(|value| value_u64(value, "durationMs")),
        Some(75)
    );

    let items = vec![json!({
        "itemId": "i~AWWayofTime~lifeEssence~0",
        "animatedAtlas": animated_atlas
    })];
    let payload = build_compact_texture_payload_from_atlas_items(&items).unwrap();
    assert_eq!(&payload[0..8], b"NEITEX1\0");
    assert_eq!(u32::from_le_bytes(payload[20..24].try_into().unwrap()), 2);
}

#[test]
fn timeline_frame_indices_wrap_to_available_exported_frames() {
    let animated_atlas = json!({
        "atlasFile": "textures/atlas/animated-main.webp",
        "frameCount": 16,
        "frameDurationMs": 50,
        "frames": [
            [0, 0, 0, 16, 16],
            [1, 16, 0, 16, 16],
            [2, 32, 0, 16, 16],
            [3, 48, 0, 16, 16]
        ],
        "timeline": [
            { "frameIndex": 0, "durationMs": 50 },
            { "frameIndex": 4, "durationMs": 50 },
            { "frameIndex": 5, "durationMs": 50 },
            { "frameIndex": 15, "durationMs": 50 }
        ]
    });

    let normalized = normalize_timeline(Some(&animated_atlas), Some(50));
    let values = normalized.as_array().expect("timeline");
    assert_eq!(values[0]["frameIndex"], json!(0));
    assert_eq!(values[1]["frameIndex"], json!(0));
    assert_eq!(values[2]["frameIndex"], json!(1));
    assert_eq!(values[3]["frameIndex"], json!(3));
}

#[test]
fn compact_string_pack_uses_native_binary_payload() {
    let items = vec![json!({
        "itemId": "minecraft:iron_ingot",
        "localizedName": "Iron Ingot",
        "modId": "minecraft",
        "internalName": "item.ingotIron",
        "groupKey": "",
        "groupLabel": "",
    })];
    let payload = build_compact_string_payload_from_items(&items).unwrap();
    assert_eq!(&payload[0..8], b"NEISTR1\0");
    assert_eq!(u32::from_le_bytes(payload[8..12].try_into().unwrap()), 1);
    assert_eq!(u32::from_le_bytes(payload[12..16].try_into().unwrap()), 1);
    assert_eq!(u32::from_le_bytes(payload[20..24].try_into().unwrap()), 6);
}

#[test]
fn compact_search_pack_uses_native_binary_payload() {
    let items = vec![json!({
        "itemId": "minecraft:iron_ingot",
        "publicItemId": "item:minecraft:iron_ingot",
        "localizedName": "Iron Ingot",
        "modId": "minecraft",
        "normalizedLocalizedName": "iron ingot",
        "normalizedInternalName": "item ingotiron",
        "normalizedItemId": "minecraft iron_ingot",
        "normalizedSearchTerms": "iron ingot minecraft item ingotiron",
        "pinyinFull": "tieding",
        "pinyinAcronym": "td",
        "popularityScore": 3,
        "searchRank": 7
    })];
    let payload = build_compact_search_payload_from_items(&items).unwrap();
    assert_eq!(&payload[0..8], b"NEISRC2\0");
    assert_eq!(u32::from_le_bytes(payload[8..12].try_into().unwrap()), 1);
    assert_eq!(u32::from_le_bytes(payload[12..16].try_into().unwrap()), 1);
    assert_eq!(u32::from_le_bytes(payload[20..24].try_into().unwrap()), 13);
}

#[test]
fn compact_recipe_pack_uses_native_binary_payload() {
    let pack = json!({
        "itemIndex": [{
            "itemId": "i~minecraft~iron_ingot~0",
            "producedBy": [{ "recipeId": "r1", "categoryId": "display~furnace", "displayName": "Furnace" }],
            "usedIn": [{ "recipeId": "r2", "categoryId": "display~crafting", "displayName": "Crafting" }]
        }],
        "uiPayloadIndex": [{
            "recipeId": "r1",
            "path": "recipes/ui-payload-shards/55.json",
            "payloadKey": "r1",
            "familyKey": "furnace",
            "recipeType": "furnace",
            "machineType": "Furnace",
            "handlerKey": "codechicken.nei.recipe.furnacerecipehandler"
        }],
        "categoryIndex": [{
            "categoryId": "display~furnace",
            "displayName": "Furnace",
            "recipeCount": 1,
            "sourceCategoryIds": ["codechicken.nei.recipe.furnacerecipehandler"],
            "machineIcon": {
                "itemId": "i~minecraft~furnace~0",
                "renderAssetRef": "nesqlpp:item/i~minecraft~furnace~0"
            }
        }]
    });
    let payload = build_compact_recipe_payload_from_pack(&pack).unwrap();
    assert_eq!(&payload[0..8], b"NEIRCP1\0");
    assert_eq!(u32::from_le_bytes(payload[8..12].try_into().unwrap()), 1);
    assert_eq!(u32::from_le_bytes(payload[16..20].try_into().unwrap()), 1);
    assert_eq!(u32::from_le_bytes(payload[20..24].try_into().unwrap()), 2);
    assert_eq!(u32::from_le_bytes(payload[24..28].try_into().unwrap()), 1);
    assert_eq!(u32::from_le_bytes(payload[28..32].try_into().unwrap()), 1);
    assert_eq!(u32::from_le_bytes(payload[36..40].try_into().unwrap()), 5);
    assert_eq!(u32::from_le_bytes(payload[40..44].try_into().unwrap()), 3);
    assert_eq!(u32::from_le_bytes(payload[44..48].try_into().unwrap()), 7);
    assert_eq!(u32::from_le_bytes(payload[48..52].try_into().unwrap()), 7);
}

#[test]
fn rust_recipe_ui_payload_paths_match_raw_export_sha1_shards() {
    assert_eq!(
        rust_recipe_ui_payload_relative_path("r1"),
        "recipes/ui-payload-shards/55.json"
    );
    assert_eq!(
        rust_recipe_ui_payload_relative_path("r~H_tVg74GOf6PmoNkwtcMLQ=="),
        "recipes/ui-payload-shards/45.json"
    );
    assert_eq!(
        rust_recipe_ui_payload_relative_path("r~prZx3D_BO22sF1-hHpwvbA=="),
        "recipes/ui-payload-shards/96.json"
    );
}

#[test]
fn captured_ui_family_key_matches_nesqlpp_census_contract() {
    let handler = json!({
        "handlerKey": "gt.recipe.assemblyline",
        "canonicalMachineFamily": "GregTech-Machine",
        "imageResource": "textures/gui/legacy.png"
    });
    let layout = json!({
        "handlerKey": "gt.recipe.assemblyline",
        "layoutKind": "Machine",
        "width": 176,
        "height": 90,
        "yShift": -4,
        "maxRecipesPerPage": 2,
        "imageResource": " textures/gui/GT5UAssemblyLine.png "
    });

    assert_eq!(
        captured_ui_family_key(Some(&handler), Some(&layout)).as_deref(),
        Some("gregtech-machine|machine|176x90@-4#2|textures/gui/gt5uassemblyline.png")
    );
}

#[test]
fn runtime_recipe_type_ids_resolve_to_nei_handler_keys() {
    let handlers = vec![json!({
        "handlerKey": "gt.recipe.laserengraver",
        "handlerClass": "gt.recipe.laserengraver",
        "canonicalMachineFamily": "gregtech-machine"
    })];
    let layouts = vec![json!({
        "handlerKey": "gt.recipe.laserengraver",
        "canonicalMachineFamily": "gregtech-machine",
        "layoutKind": "machine",
        "width": 166,
        "height": 135,
        "maxRecipesPerPage": 2,
        "progressBars": [{ "x": 78, "y": 24, "width": 20, "height": 18 }]
    })];
    let recipe = json!({
        "family": "gregtech",
        "sourcePlugin": "gregtech",
        "machine": {
            "machineId": "rt~gregtech~gt.recipe.laserengraver~MV",
            "displayName": "gregtech - Laser Engraver (MV)"
        }
    });
    let context = RecipeHandlerContext::new(&handlers, &layouts);

    let (handler, layout) = context.resolve(&recipe);

    assert_eq!(
        handler
            .and_then(|value| value_string(value, "handlerKey"))
            .as_deref(),
        Some("gt.recipe.laserengraver")
    );
    assert_eq!(
        captured_ui_family_key(handler, layout).as_deref(),
        Some("gregtech-machine|machine|166x135@0#2|unknown")
    );
}

#[test]
fn public_recipe_layout_preserves_native_background_and_dynamic_primitives() {
    let layout = json!({
        "handlerKey": "gt.recipe.assemblyline",
        "canonicalMachineFamily": "gregtech-machine",
        "layoutKind": "machine",
        "width": 176,
        "height": 90,
        "imageResource": "textures/gui/gt5u_assembly_line.png",
        "imageRegion": { "x": 4, "y": 8, "width": 176, "height": 90 },
        "nativeBackground": {
            "status": "captured",
            "kind": "gt-modular-ui",
            "assetRef": "assets/ui-backgrounds/gregtech/nei_single_recipe.png",
            "scaling": "nine-slice"
        },
        "progressBars": [{
            "kind": "progress-bar",
            "role": "gt-progress",
            "x": 78,
            "y": 24,
            "width": 20,
            "height": 18
        }],
        "dynamicPrimitives": [{
            "kind": "progress-bar",
            "x": 78,
            "y": 24,
            "width": 20,
            "height": 18
        }],
        "hotspots": [{
            "id": "machine-info",
            "label": "Machine info",
            "x": 6,
            "y": 6,
            "width": 48,
            "height": 12
        }],
        "viewports": [{
            "id": "preview",
            "kind": "item-preview",
            "x": 120,
            "y": 8,
            "width": 32,
            "height": 32
        }]
    });

    let public_layout = public_recipe_layout(&layout);

    assert_eq!(
        public_layout["imageResource"],
        json!("textures/gui/gt5u_assembly_line.png")
    );
    assert_eq!(
        public_layout["canonicalMachineFamily"],
        json!("gregtech-machine")
    );
    assert_eq!(public_layout["imageRegion"]["x"], json!(4));
    assert_eq!(
        public_layout["nativeBackground"]["assetRef"],
        json!("assets/ui-backgrounds/gregtech/nei_single_recipe.png")
    );
    assert_eq!(
        public_layout["progressBars"].as_array().unwrap()[0]["role"],
        json!("gt-progress")
    );
    assert_eq!(
        public_layout["dynamicPrimitives"].as_array().unwrap()[0]["width"],
        json!(20)
    );
    assert_eq!(
        public_layout["hotspots"].as_array().unwrap()[0]["label"],
        json!("Machine info")
    );
    assert_eq!(
        public_layout["viewports"].as_array().unwrap()[0]["kind"],
        json!("item-preview")
    );
}

#[test]
fn ui_template_bindings_use_captured_family_keys_not_simple_recipe_families() {
    let captured_family_key =
        "gregtech-machine|machine|176x90@-4#2|textures/gui/gt5uassemblyline.png";
    let templates = vec![json!({
        "templateKey": "ui-template/assembly-line",
        "templateSignature": "assemblyline123",
        "familyKey": captured_family_key,
        "canonicalMachineFamily": "gregtech-machine",
        "layoutKind": "machine"
    })];
    let recipe_index = vec![json!({
        "recipeId": "r_gt_assembly_line",
        "familyKey": captured_family_key,
        "recipeType": "gt.recipe.assemblyline",
        "machineType": "Assembly Line"
    })];

    let bindings = build_ui_template_bindings(&recipe_index, &templates);

    assert_eq!(
        bindings[0]["templateKey"],
        json!("ui-template/assembly-line")
    );
    assert_eq!(bindings[0]["familyKey"], json!(captured_family_key));
}

#[test]
fn ui_assets_manifest_and_materializer_include_native_background_assets() {
    let templates = vec![json!({
        "templateKey": "gt-machine@default",
        "imageResource": "",
        "nativeBackground": {
            "status": "captured",
            "kind": "gt-modular-ui",
            "assetRef": "assets/ui-backgrounds/gregtech/nei_single_recipe.png",
            "scaling": "nine-slice"
        }
    })];
    let manifest = build_ui_assets_manifest(&templates);
    assert_eq!(manifest["assets"].as_array().unwrap().len(), 1);
    assert_eq!(
        manifest["assets"][0]["assetRef"],
        json!("assets/ui-backgrounds/gregtech/nei_single_recipe.png")
    );

    let input = tempfile::tempdir().unwrap();
    let output = tempfile::tempdir().unwrap();
    let source = input
        .path()
        .join("assets/ui-backgrounds/gregtech/nei_single_recipe.png");
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::write(&source, b"png").unwrap();

    let materialized =
        materialize_ui_background_assets(input.path(), output.path(), &manifest).unwrap();

    assert_eq!(
        fs::read(
            output
                .path()
                .join("assets/ui-backgrounds/gregtech/nei_single_recipe.png")
        )
        .unwrap(),
        b"png"
    );
    assert_eq!(materialized["copied"].as_array().unwrap().len(), 1);
    assert_eq!(materialized["missing"].as_array().unwrap().len(), 0);
}

#[test]
fn compact_ui_pack_uses_shared_native_string_table() {
    let templates = vec![json!({
        "templateKey": "furnace@default",
        "templateSignature": "abc123",
        "familyKey": "furnace",
        "canonicalMachineFamily": "furnace",
        "layoutKind": "furnace",
        "width": 166,
        "height": 65,
        "yShift": -4,
        "maxRecipesPerPage": 2,
        "imageResource": "textures/gui/furnace.png",
        "handlerCount": 1,
        "slots": [
            { "role": "item-input", "startIndex": 0, "columns": 1, "rows": 1, "x": 45, "y": 24 },
            { "role": "item-output", "startIndex": 1, "columns": 1, "rows": 1, "x": 115, "y": 24 }
        ],
        "textOverlays": [{ "text": "EU/t", "x": 80, "y": 10, "width": 24, "height": 8 }]
    })];
    let recipe_index = vec![json!({
        "recipeId": "r1",
        "path": "recipes/ui-payload-shards/55.json",
        "payloadKey": "r1",
        "familyKey": "furnace",
        "recipeType": "furnace",
        "machineType": "Furnace"
    })];
    let bindings = build_ui_template_bindings(&recipe_index, &templates);
    let mut strings = vec![String::new()];
    let mut string_refs = HashMap::new();
    string_refs.insert(String::new(), 0u32);
    let template_payload =
        build_compact_ui_template_payload(&templates, &mut strings, &mut string_refs).unwrap();
    let binding_payload =
        build_compact_ui_binding_payload(&bindings, &mut strings, &mut string_refs).unwrap();
    let string_payload = build_compact_ui_string_payload(&strings).unwrap();

    assert_eq!(&template_payload[0..8], b"NEIUIT1\0");
    assert_eq!(
        u32::from_le_bytes(template_payload[8..12].try_into().unwrap()),
        3
    );
    assert_eq!(
        u32::from_le_bytes(template_payload[12..16].try_into().unwrap()),
        1
    );
    assert_eq!(
        u32::from_le_bytes(template_payload[16..20].try_into().unwrap()),
        2
    );
    assert_eq!(
        u32::from_le_bytes(template_payload[20..24].try_into().unwrap()),
        1
    );
    assert_eq!(
        u32::from_le_bytes(template_payload[24..28].try_into().unwrap()),
        0
    );
    assert_eq!(
        u32::from_le_bytes(template_payload[28..32].try_into().unwrap()),
        0
    );
    assert_eq!(
        u32::from_le_bytes(template_payload[32..36].try_into().unwrap()),
        19
    );
    assert_eq!(
        u32::from_le_bytes(template_payload[44..48].try_into().unwrap()),
        12
    );
    assert_eq!(&binding_payload[0..8], b"NEIUIB1\0");
    assert_eq!(
        u32::from_le_bytes(binding_payload[12..16].try_into().unwrap()),
        1
    );
    assert_eq!(&string_payload[0..8], b"NEIUIS1\0");
    assert!(u32::from_le_bytes(string_payload[12..16].try_into().unwrap()) > 8);
    assert_eq!(bindings[0]["templateKey"], json!("furnace@default"));
}

#[test]
fn zero_recipe_diagnostics_distinguish_legal_and_suspicious_handlers() {
    let value = json!({
        "summary": {
            "status": "warning",
            "totalHandlers": 8,
            "handlersWithLoadedRecipes": 6,
            "handlersWithExportedRecipes": 4,
            "expectedEmptyHandlers": 2,
            "nativeCoveredZeroExports": 3,
            "nonRecipeInfoZeroExports": 1,
            "suspiciousZeroExports": 1,
            "partialExports": 2
        }
    });
    let diagnostics = raw_export::zero_recipe_diagnostics_from_value(&value).unwrap();
    assert_eq!(diagnostics.status.as_deref(), Some("warning"));
    assert_eq!(diagnostics.legal_zero_recipe_handlers, 6);
    assert_eq!(diagnostics.suspicious_zero_exports, 1);
    assert_eq!(diagnostics.partial_exports, 2);
}

#[test]
fn native_ui_layout_report_counts_gregtech_progress_and_backgrounds() {
    let temp = tempfile::tempdir().unwrap();
    let recipes_dir = temp.path().join("recipes");
    fs::create_dir_all(&recipes_dir).unwrap();
    write_json_value(
        &recipes_dir.join("handler-layout-index.json"),
        &json!({
            "schemaVersion": "neonei/recipe-handler-layout-index/v1",
            "layouts": [{
                "handlerKey": "gt.recipe.test",
                "handlerClass": "gregtech.nei.GTNEIDefaultHandler",
                "canonicalMachineFamily": "gregtech-machine",
                "layoutKind": "machine",
                "width": 176,
                "height": 90,
                "maxRecipesPerPage": 1,
                "imageRegion": { "x": 0, "y": 0, "width": 176, "height": 90 },
                "nativeBackground": {
                    "status": "captured",
                    "kind": "gt-modular-ui",
                    "assetRef": "assets/ui-backgrounds/gregtech/nei_single_recipe.png",
                    "resource": "gregtech:textures/gui/background/nei_single_recipe.png",
                    "scaling": "nine-slice",
                    "texture": { "width": 64, "height": 64, "borderU": 2, "borderV": 2 }
                },
                "progressBars": [{ "x": 78, "y": 24, "width": 20, "height": 18 }]
            }]
        }),
    )
    .unwrap();
    write_json_value(
        &recipes_dir.join("ui-payload-index.json"),
        &json!({
            "schemaVersion": "neonei/recipe-ui-payload-index/v1",
            "recipes": [{
                "recipeId": "gt:test",
                "familyKey": "gregtech-machine|machine|176x90@0#1|unknown",
                "nativeLayout": {
                    "canonicalMachineFamily": "gregtech-machine",
                    "imageRegion": { "x": 0, "y": 0, "width": 176, "height": 90 },
                    "progressBars": [{ "x": 78, "y": 24, "width": 20, "height": 18 }]
                }
            }]
        }),
    )
    .unwrap();

    let report =
        native_ui_report::compile_native_ui_layout_report(temp.path(), captured_ui_family_key)
            .unwrap()
            .unwrap();

    assert_eq!(report["status"], json!("ready"));
    assert_eq!(report["counts"]["gregtechHandlerLayouts"], json!(1));
    assert_eq!(
        report["counts"]["gregtechRecipeUiPayloadsWithProgressBars"],
        json!(1)
    );
    assert_eq!(report["backgroundStatus"], json!("captured"));
    assert_eq!(
        report["counts"]["gregtechRecipeUiPayloadsWithNativeBackgrounds"],
        json!(1)
    );
    assert!(temp
        .path()
        .join("rust")
        .join("native-ui-layout-report.json")
        .exists());
}

#[test]
fn production_manifest_entries_exclude_debug_json_packs() {
    let production_entries = runtime::rust_manifest_file_entries(CompileScope::All, false)
        .into_iter()
        .map(|(_, path)| path)
        .collect::<Vec<_>>();
    assert!(production_entries.contains(&"rust/browser.bin"));
    assert!(production_entries.contains(&"rust/search.bin"));
    assert!(production_entries.contains(&"rust/recipes.bin"));
    assert!(production_entries.contains(&"rust/textures.bin"));
    assert!(production_entries.contains(&"rust/atlas.meta.bin"));
    assert!(production_entries.contains(&"rust/ui-pack/ui_templates.bin"));
    assert!(production_entries.contains(&"rust/ui-pack/ui_bindings.bin"));
    assert!(production_entries.contains(&"rust/ui-pack/ui_strings.bin"));
    assert!(production_entries.contains(&"rust/ui-pack/ui_assets.manifest.json"));
    assert!(production_entries.contains(&"rust/ui-pack/ui_pack_report.json"));
    assert!(production_entries.contains(&"rust/native-ui-layout-report.json"));
    assert!(production_entries.contains(&"rust/semantic-validation-report.json"));
    assert!(production_entries.contains(&"rust/recipe-handler-metadata-report.json"));
    assert!(production_entries.contains(&"rust/recipe-fragmentation-report.json"));
    assert!(!production_entries.contains(&"rust/browser-pack.json"));
    assert!(!production_entries.contains(&"rust/search-pack.json"));
    assert!(!production_entries.contains(&"rust/recipe-pack.json"));
    assert!(!production_entries.contains(&"rust/texture-pack.json"));

    let debug_entries = runtime::rust_manifest_file_entries(CompileScope::All, true)
        .into_iter()
        .map(|(_, path)| path)
        .collect::<Vec<_>>();
    assert!(debug_entries.contains(&"rust/browser-pack.json"));
    assert!(debug_entries.contains(&"rust/search-pack.json"));
    assert!(debug_entries.contains(&"rust/recipe-pack.json"));
    assert!(debug_entries.contains(&"rust/texture-pack.json"));
}

#[test]
fn stable_cli_inspect_validate_and_schemas_cover_fixture_contracts() {
    let temp = tempfile::tempdir().unwrap();
    let raw = compiler_fixture_path("raw-export-minimal");
    let inspect_report = temp.path().join("inspect-report.json");
    let validate_report = temp.path().join("validate-report.json");
    let schema_catalog = temp.path().join("schemas/catalog.json");

    run_command(Cli {
        command: Command::Inspect {
            input: raw.clone(),
            report: inspect_report.clone(),
            threads: Some(1),
        },
    })
    .unwrap();
    run_command(Cli {
        command: Command::Validate {
            input: raw,
            report: validate_report.clone(),
            output: None,
            threads: Some(1),
        },
    })
    .unwrap();
    run_command(Cli {
        command: Command::Schemas {
            output: Some(schema_catalog.clone()),
        },
    })
    .unwrap();

    let inspect: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(inspect_report).unwrap()).unwrap();
    let validate: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(validate_report).unwrap()).unwrap();
    let schemas: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(schema_catalog).unwrap()).unwrap();

    assert_eq!(inspect["mode"], json!("inspect"));
    assert_eq!(validate["mode"], json!("validate"));
    assert!(validate["blocked"].as_array().unwrap().is_empty());
    assert_eq!(
        schemas["schemaVersion"],
        json!("elysium-compiler/schema-catalog/v1")
    );
    assert!(schemas["compiler"]["cli"]["commands"]
        .as_array()
        .unwrap()
        .contains(&json!("schemas")));
    assert_eq!(
        schemas["rawExport"]["nativeBackground"]["strictPolicy"],
        json!("a captured nativeBackground.assetRef must point to a materialized raw-export asset")
    );
}

#[test]
fn minimal_native_ui_fixture_compiles_through_stable_cli_boundary() {
    let output = compile_fixture("raw-export-minimal", CompileScope::NativeUi, false);
    let report = output.path().join("compiler-report.json");

    assert!(report.exists());
    assert!(output.path().join("rust/runtime-manifest.json").exists());
    assert!(output.path().join("rust/browser.bin").exists());
    assert!(output.path().join("rust/recipes.bin").exists());
    assert!(output.path().join("rust/ui-pack/ui_templates.bin").exists());
    assert!(output
        .path()
        .join("assets/ui-backgrounds/gregtech/nei_single_recipe.png")
        .exists());

    let runtime_manifest: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(output.path().join("rust/runtime-manifest.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(runtime_manifest["compileScope"], json!("native-ui"));
    assert_eq!(
        runtime_manifest["schemaVersion"],
        json!("neonei/rust-runtime-manifest/current")
    );
    assert!(runtime_manifest["files"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["path"] == json!("rust/ui-pack/ui_assets.manifest.json")));
    assert!(runtime_manifest["files"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["path"] == json!("rust/ui-pack/ui_template_catalog.json")));
    assert!(runtime_manifest["files"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["path"] == json!("rust/ui-pack/ui_template_binding_index.json")));
    assert!(runtime_manifest["files"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["path"] == json!("rust/ui-pack/ui_family_census.json")));

    let ui_pack_report: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(output.path().join("rust/ui-pack/ui_pack_report.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(ui_pack_report["status"], json!("ready"));
    assert_eq!(ui_pack_report["summary"]["templateCount"], json!(1));
    assert_eq!(
        ui_pack_report["assets"]["uiBackgrounds"]["missing"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    let ui_template_catalog =
        read_fixture_json(output.path().join("rust/ui-pack/ui_template_catalog.json"));
    let ui_binding_index = read_fixture_json(
        output
            .path()
            .join("rust/ui-pack/ui_template_binding_index.json"),
    );
    let ui_family_census =
        read_fixture_json(output.path().join("rust/ui-pack/ui_family_census.json"));
    assert_eq!(
        ui_template_catalog["schemaVersion"],
        json!("neonei/ui-template-catalog/current")
    );
    assert_eq!(
        ui_binding_index["schemaVersion"],
        json!("neonei/ui-template-binding-index/current")
    );
    assert_eq!(
        ui_family_census["schemaVersion"],
        json!("neonei/ui-family-census/current")
    );
}

#[test]
fn native_ui_gt_fixture_matches_expected_reports_and_copies_background_asset() {
    let output = compile_fixture("raw-export-native-ui-gt", CompileScope::NativeUi, true);
    for relative_path in [
        "recipes/ui-payload-index.json",
        "rust/runtime-manifest.json",
        "rust/native-ui-layout-report.json",
        "rust/ui-pack/ui_pack_report.json",
        "rust/ui-pack/ui_assets.manifest.json",
        "rust/ui-pack/ui_template_catalog.json",
        "rust/ui-pack/ui_template_binding_index.json",
        "rust/ui-pack/ui_family_census.json",
        "rust/integrity.json",
    ] {
        assert_expected_json_matches("raw-export-native-ui-gt", output.path(), relative_path);
    }
    assert!(output
        .path()
        .join("assets/ui-backgrounds/gregtech/nei_single_recipe.png")
        .is_file());
}

#[test]
fn semantic_background_only_fixture_compiles_without_materialized_asset() {
    let output = compile_fixture(
        "raw-export-semantic-background-only",
        CompileScope::NativeUi,
        true,
    );
    let layout_report = read_fixture_json(output.path().join("rust/native-ui-layout-report.json"));
    let ui_assets = read_fixture_json(output.path().join("rust/ui-pack/ui_assets.manifest.json"));
    let ui_pack_report = read_fixture_json(output.path().join("rust/ui-pack/ui_pack_report.json"));

    assert_eq!(layout_report["backgroundStatus"], json!("semantic"));
    assert_eq!(layout_report["status"], json!("ready"));
    assert!(ui_assets["assets"].as_array().unwrap().is_empty());
    assert!(ui_pack_report["assets"]["uiBackgrounds"]["missing"]
        .as_array()
        .unwrap()
        .is_empty());
    for relative_path in [
        "recipes/ui-payload-index.json",
        "rust/runtime-manifest.json",
        "rust/native-ui-layout-report.json",
        "rust/ui-pack/ui_pack_report.json",
        "rust/ui-pack/ui_assets.manifest.json",
        "rust/ui-pack/ui_template_catalog.json",
        "rust/ui-pack/ui_template_binding_index.json",
        "rust/ui-pack/ui_family_census.json",
        "rust/integrity.json",
    ] {
        assert_expected_json_matches(
            "raw-export-semantic-background-only",
            output.path(),
            relative_path,
        );
    }
}

#[test]
fn sharded_recipes_fixture_compiles_all_declared_shards() {
    let output = compile_fixture("raw-export-sharded-recipes", CompileScope::NativeUi, true);
    let ui_payload_index = read_fixture_json(output.path().join("recipes/ui-payload-index.json"));
    let recipes = ui_payload_index["recipes"].as_array().unwrap();
    assert_eq!(recipes.len(), 2);
    assert!(recipes
        .iter()
        .any(|entry| entry["recipeId"] == json!("r_fixture_shard_a")));
    assert!(recipes
        .iter()
        .any(|entry| entry["recipeId"] == json!("r_fixture_shard_b")));
    assert!(recipes
        .iter()
        .all(|entry| entry["nativeLayout"]["progressBars"]
            .as_array()
            .is_some_and(|bars| !bars.is_empty())));
    assert_expected_json_matches(
        "raw-export-sharded-recipes",
        output.path(),
        "recipes/ui-payload-index.json",
    );
    for relative_path in [
        "rust/ui-pack/ui_template_catalog.json",
        "rust/ui-pack/ui_template_binding_index.json",
        "rust/ui-pack/ui_family_census.json",
    ] {
        assert_expected_json_matches("raw-export-sharded-recipes", output.path(), relative_path);
    }
}

#[test]
fn texture_atlas_fixture_materializes_runtime_atlas_without_missing_refs() {
    let output = compile_fixture("raw-export-texture-atlas", CompileScope::All, true);
    let missing_texture_report =
        read_fixture_json(output.path().join("rust/missing-texture-report.json"));
    assert_eq!(missing_texture_report["status"], json!("ok"));
    assert_eq!(
        missing_texture_report["counts"]["missingAtlasAssetFiles"],
        json!(0)
    );
    assert!(output
        .path()
        .join("textures/atlas/static-fixture.webp")
        .is_file());
    for relative_path in [
        "recipes/ui-payload-index.json",
        "rust/runtime-manifest.json",
        "rust/missing-texture-report.json",
        "rust/suspicious-texture-report.json",
        "rust/ui-pack/ui_template_catalog.json",
        "rust/ui-pack/ui_template_binding_index.json",
        "rust/ui-pack/ui_family_census.json",
    ] {
        assert_expected_json_matches("raw-export-texture-atlas", output.path(), relative_path);
    }
}

#[test]
fn missing_captured_ui_background_fixture_fails_strict_compile() {
    let output = tempfile::tempdir().unwrap();
    let report = output.path().join("compiler-report.json");
    let raw = compiler_fixture_path("raw-export-missing-background-should-fail");

    let error = run_command(Cli {
        command: Command::Compile {
            input: raw,
            output: output.path().to_path_buf(),
            report,
            scope: CompileScope::NativeUi,
            threads: Some(1),
            strict: true,
            debug_json: false,
        },
    })
    .expect_err(
        "strict compile must fail when a captured UI background asset is declared but missing",
    );

    let message = format!("{error:#}");
    assert!(
        message.contains("ui-pack compiler blocked")
            && message.contains("native UI background asset")
            && message.contains("missing"),
        "unexpected error: {message}"
    );
    assert!(!output.path().join("rust/runtime-manifest.json").exists());
}
