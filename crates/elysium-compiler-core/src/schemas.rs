use crate::abi::abi_catalog;
use crate::io::write_json_value;
use crate::version::{
    metadata as compiler_metadata, COMPILED_DIST_SCHEMA_VERSION, RAW_EXPORT_SCHEMA_VERSION,
};
use anyhow::Result;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

pub fn schema_catalog() -> Value {
    json!({
        "schemaVersion": "elysium-compiler/schema-catalog/v1",
        "abi": abi_catalog(),
        "compiler": {
            "name": "elysium-compiler",
            "currentCrate": "elysium-compiler-core",
            "metadata": compiler_metadata(),
            "cli": {
                "commands": ["compile", "inspect", "validate", "schemas"],
                "compileScopes": ["all", "native-ui", "search", "browser", "recipes", "ui", "textures"]
            }
        },
        "rawExport": {
            "schemaVersion": RAW_EXPORT_SCHEMA_VERSION,
            "manifest": {
                "schemaVersion": "neonei/raw-export-manifest/current",
                "requiredFiles": ["items", "fluids", "recipeIndex", "browserAtlasIndex"],
                "optionalFiles": [
                    "neiOrder",
                    "browserGroups",
                    "textures",
                    "animations",
                    "nativeSprites",
                    "neiHandlers",
                    "neiHandlerLayouts",
                    "uiTemplateCatalog",
                    "exportReport",
                    "neiBrowserContract",
                    "semanticFamilyAudit",
                    "semanticNbtKeyDistribution",
                    "semanticIdentityNormalizationReport",
                    "neiHandlerAnomalies"
                ],
                "pathPolicy": "portable-relative-only; no drive letters, file URLs, absolute paths, '.', or '..' segments"
            },
            "nativeBackground": {
                "requiredCapturedFields": ["status", "assetRef"],
                "capturedStatus": "captured",
                "assetRoot": "assets/ui-backgrounds/",
                "scaling": ["nine-slice", "stretch", "none"],
                "strictPolicy": "a captured nativeBackground.assetRef must point to a materialized raw-export asset"
            }
        },
        "distData": {
            "schemaVersion": COMPILED_DIST_SCHEMA_VERSION,
            "manifest": "neonei/dist-data/current",
            "runtimeManifest": "neonei/rust-runtime-manifest/current",
            "packValidationReport": "elysium-compiler/pack-abi-validation/v1",
            "runtimeEntrypoints": [
                "rust/browser.bin",
                "rust/groups.bin",
                "rust/search.bin",
                "rust/recipes.bin",
                "rust/textures.bin",
                "rust/atlas.meta.bin",
                "rust/animations.bin",
                "rust/strings.zh_cn.bin",
                "rust/ui-pack/ui_templates.bin",
                "rust/ui-pack/ui_bindings.bin",
                "rust/ui-pack/ui_strings.bin"
            ],
            "uiPack": {
                "assetsManifest": "neonei/ui-assets-manifest/current",
                "report": "neonei/ui-pack-report/current",
                "templatePackSchema": "neonei/ui-template-pack/current",
                "bindingPackSchema": "neonei/ui-binding-pack/current",
                "stringPackSchema": "neonei/ui-string-pack/current"
            },
            "recipeUiPayloadIndex": "neonei/recipe-ui-payload-index/v1"
        }
    })
}

pub fn write_schema_catalog(output: Option<&Path>) -> Result<()> {
    let catalog = schema_catalog();
    if let Some(path) = output {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        write_json_value(path, &catalog)
    } else {
        println!("{}", serde_json::to_string_pretty(&catalog)?);
        Ok(())
    }
}
