use elysium_compiler_core::domain::Probe;
use elysium_compiler_core::identity::{fluid_id, item_id, property_key, registry_name, Nbt};
use elysium_compiler_core::source::{source_path, Source, SourceManifest};
use serde_json::json;
use std::fs;
use std::path::PathBuf;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures/source")
}

#[test]
fn java_source_is_verified_without_losing_identity_or_large_quantities() {
    let source = Source::open(&fixture()).unwrap();
    source.verify().unwrap();
    let probes = source.environment().unwrap().probes;
    assert_eq!(probes.len(), 2);
    assert_eq!(probes[0].count, 1);
    assert_eq!(probes[1].count, 4);
    assert_eq!(probes[1].channels["coil"], 2);
    source
        .visit("items", |item| {
            let nbt: Option<Nbt> = serde_json::from_value(item["nbt"].clone()).unwrap();
            assert_eq!(
                item["id"],
                item_id(
                    item["registry"].as_str().unwrap(),
                    item["meta"].as_i64().unwrap() as i32,
                    nbt.as_ref()
                )
                .unwrap()
            );
            assert!(item["name"].as_str().unwrap().starts_with("text_"));
            Ok(())
        })
        .unwrap();
    source
        .visit("fluids", |fluid| {
            let nbt: Option<Nbt> = serde_json::from_value(fluid["nbt"].clone()).unwrap();
            assert_eq!(
                fluid["id"],
                fluid_id(fluid["registry"].as_str().unwrap(), nbt.as_ref()).unwrap()
            );
            Ok(())
        })
        .unwrap();
    source
        .visit("recipes", |recipe| {
            if recipe["source"]["key"] == "scanner" {
                assert_eq!(recipe["outputs"][0]["change"]["action"]["kind"], "analyze");
                assert_eq!(
                    recipe["inputs"][0]["choices"][0]["consume"]["kind"],
                    "stack"
                );
                assert!(recipe["magic"].is_null());
                return Ok(());
            }
            if recipe["source"]["key"] != "machine" {
                if recipe["source"]["key"] == "wand_creative" {
                    assert_eq!(recipe["magic"]["creative"], true);
                    assert!(recipe["magic"]["aspects"].as_array().unwrap().is_empty());
                    return Ok(());
                }
                let changed = recipe["outputs"][0]["change"].is_object();
                assert_eq!(
                    recipe["magic"]["aspects"][0]["amount"],
                    if changed { "2" } else { "7" }
                );
                assert!(recipe["inputs"][0]["choices"][0]["rule"].is_object());
                return Ok(());
            }
            assert_eq!(
                recipe["inputs"][0]["choices"][0]["amount"],
                "9007199254740993"
            );
            assert_eq!(recipe["energy"], "9223372036854775807");
            Ok(())
        })
        .unwrap();
    let nbt: Nbt = serde_json::from_value(json!({"type":"compound","value":{
        "long":{"type":"long","value":"9223372036854775807"},
        "float":{"type":"float","value":"80000000"},
        "list":{"type":"list","element":"int","value":[{"type":"int","value":"7"},{"type":"int","value":"-4"}]}
    }})).unwrap();
    nbt.validate().unwrap();
    assert_ne!(
        item_id("minecraft:stone", 0, Some(&nbt)).unwrap(),
        item_id("minecraft:stone", 0, None).unwrap()
    );
    for (registry, expected) in [
        (
            "BiblioCraft:Armor Stand",
            "item_49eeed3f0c9fd63f329598282a1406c58accbe6729a6195ebef95e6c4dcd84c1",
        ),
        (
            "ProjRed|Core:projectred.core.part",
            "item_cc00b969a3e38a4bd9e7c34a9a4ce0d01befa1703559b3ed4b06a95031801347",
        ),
        (
            "兼容|测试:方块 \"A\" /β",
            "item_ef3cfffdb74e2147983247b80182807f3ed49de488c2837dcc8d9e22e5f40d12",
        ),
        (
            "owner:part:variant",
            "item_03e51f074fd995a201649bab53fc67bc0e92857f6598cc2692bb981dd02b37b0",
        ),
    ] {
        registry_name(registry).unwrap();
        assert_eq!(item_id(registry, 0, None).unwrap(), expected);
        assert!(property_key(registry).is_err());
        assert!(source_path(registry).is_err());
    }
    let variants: std::collections::BTreeSet<_> = [
        "BiblioCraft:Armor Stand",
        "BiblioCraft:Armor_Stand",
        "bibliocraft:Armor Stand",
        "BiblioCraft:Armor Stand ",
    ]
    .iter()
    .map(|registry| item_id(registry, 0, None).unwrap())
    .collect();
    assert_eq!(variants.len(), 4);
    assert_eq!(
        fluid_id("Liquid Crystal", None).unwrap(),
        "fluid_3e9abc09ea399c31f244541b5db47c173b8f1f1eaaef1da2ac06eab0ab773ca6"
    );
    for invalid in ["", "stone", ":stone", "minecraft:"] {
        assert!(registry_name(invalid).is_err());
        assert!(item_id(invalid, 0, None).is_err());
    }
    assert!(fluid_id("", None).is_err());
    property_key("gt:heat").unwrap();
}

#[test]
fn source_rejects_corruption_unknown_revisions_and_undeclared_paths() {
    for value in [
        json!([]),
        json!([{"count": 0, "channels": {}}]),
        json!([{"count": 65, "channels": {}}]),
        json!([{"count": 1.5, "channels": {}}]),
        json!([{"count": "1", "channels": {}}]),
        json!([{"count": 1, "channels": {"Coil": 2}}]),
        json!([{"count": 1, "channels": {"coil": 0}}]),
        json!([{"count": 1, "channels": {"coil": 65536}}]),
        json!([{"count": 1, "channels": {"coil": "2"}}]),
        json!([{"count": 1, "channels": {}}, {"count": 1, "channels": {}}]),
        json!([{"count": 10, "channels": {}}, {"count": 2, "channels": {}}]),
    ] {
        assert!(
            serde_json::from_value::<Vec<Probe>>(value.clone())
                .map_err(anyhow::Error::from)
                .and_then(|probes| Probe::validate_all(&probes))
                .is_err(),
            "accepted invalid probes: {value}"
        );
    }
    let source = Source::open(&fixture()).unwrap();
    let mut manifest = source.manifest.clone();
    manifest.revision = 11;
    manifest.id = manifest.digest().unwrap();
    assert!(manifest
        .validate()
        .unwrap_err()
        .to_string()
        .contains("unsupported source revision"));
    let mut manifest = source.manifest.clone();
    manifest.revision += 1;
    manifest.id = manifest.digest().unwrap();
    assert!(manifest
        .validate()
        .unwrap_err()
        .to_string()
        .contains("unsupported source revision"));
    assert!(
        serde_json::from_value::<SourceManifest>(json!({"schemaVersion":"old","files":{}}))
            .is_err()
    );
    let directory = tempfile::tempdir().unwrap();
    fs::copy(
        fixture().join("manifest.json"),
        directory.path().join("manifest.json"),
    )
    .unwrap();
    let file = &source.manifest.files[0];
    let target = directory.path().join(&file.path);
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    let mut bytes = fs::read(fixture().join(&file.path)).unwrap();
    bytes[0] ^= 1;
    fs::write(target, bytes).unwrap();
    let damaged = Source::open(directory.path()).unwrap();
    assert!(damaged
        .read_file(file)
        .unwrap_err()
        .to_string()
        .contains("digest mismatch"));
    for value in [
        "../secret",
        "C:/secret",
        "/secret",
        "assets\\a.png",
        "assets/CON.png",
        "assets/x.",
        "assets/%2e%2e/x",
    ] {
        assert!(source_path(value).is_err(), "accepted {value}");
    }
}
