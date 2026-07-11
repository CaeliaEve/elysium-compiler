//! Descriptor-owned schema catalog sections.
//!
//! Keep the public `schemas` command thin: raw-export ABI, dist-data ABI,
//! report schema paths, and runtime hot-path declarations are declared here as
//! static catalog descriptors and validated before projection. This follows the
//! kernel-style rule that ABI surfaces are tables first and procedural JSON
//! assembly second.

use crate::manifest::{manifest_collection_catalog, RAW_MANIFEST_PATH_POLICY};
use crate::native_ui_export_abi_catalog::{
    NATIVE_UI_EXPORT_ABI_VALIDATION_REPORT_PATH, NATIVE_UI_EXPORT_ABI_VALIDATION_SCHEMA_VERSION,
    NATIVE_UI_VALIDATION_DEFAULT_PATH, NESQL_NATIVE_UI_VALIDATION_SCHEMA_VERSION,
};
use crate::pack_abi::{runtime_artifact_catalog, PACK_ABI_VALIDATION_SCHEMA_VERSION};
use crate::raw_export_abi::RAW_EXPORT_ABI_VALIDATION_SCHEMA_VERSION;
use crate::runtime_manifest_abi::{RUST_RUNTIME_ENTRYPOINTS, RUST_RUNTIME_MANIFEST_SCHEMA_VERSION};
use crate::ui_pack_abi::{
    UI_PACK_ABI_VALIDATION_REPORT_PATH, UI_PACK_ABI_VALIDATION_SCHEMA_VERSION,
};
use crate::version::{COMPILED_DIST_SCHEMA_VERSION, RAW_EXPORT_SCHEMA_VERSION};
use serde_json::{json, Map, Value};
use std::collections::BTreeSet;

pub const SCHEMA_CATALOG_SCHEMA_VERSION: &str = "elysium-compiler/schema-catalog/v1";
const RAW_EXPORT_MANIFEST_SCHEMA_VERSION: &str = "neonei/raw-export-manifest/current";
const RAW_EXPORT_VALIDATION_POLICY: &str =
    "missing required files, missing declared files, path violations, and legacy fallback are compile blockers";
const NATIVE_UI_EXPORT_VALIDATION_POLICY: &str =
    "missing native UI validation, blocked geometry reports, and schema mismatches are compile blockers";
const NATIVE_BACKGROUND_STRICT_POLICY: &str =
    "nativeBackground is semantic layout metadata only; background PNG assets are retired and are not materialized";
const NATIVE_FRAME_STRICT_POLICY: &str =
    "nativeFrame PNG capture is retired; recipe UI payloads must not depend on in-game NEI frame assets";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RawExportManifestFileDescriptor {
    pub key: &'static str,
    pub required: bool,
}

impl RawExportManifestFileDescriptor {
    const fn required(key: &'static str) -> Self {
        Self {
            key,
            required: true,
        }
    }

    const fn optional(key: &'static str) -> Self {
        Self {
            key,
            required: false,
        }
    }
}

pub const RAW_EXPORT_MANIFEST_FILE_DESCRIPTORS: &[RawExportManifestFileDescriptor] = &[
    RawExportManifestFileDescriptor::required("items"),
    RawExportManifestFileDescriptor::required("fluids"),
    RawExportManifestFileDescriptor::required("recipeIndex"),
    RawExportManifestFileDescriptor::required("browserAtlasIndex"),
    RawExportManifestFileDescriptor::optional("neiOrder"),
    RawExportManifestFileDescriptor::optional("browserGroups"),
    RawExportManifestFileDescriptor::optional("textures"),
    RawExportManifestFileDescriptor::optional("animations"),
    RawExportManifestFileDescriptor::optional("nativeSprites"),
    RawExportManifestFileDescriptor::optional("neiHandlers"),
    RawExportManifestFileDescriptor::optional("neiHandlerLayouts"),
    RawExportManifestFileDescriptor::optional("uiTemplateCatalog"),
    RawExportManifestFileDescriptor::optional("nativeUiValidation"),
    RawExportManifestFileDescriptor::optional("exportReport"),
    RawExportManifestFileDescriptor::optional("neiBrowserContract"),
    RawExportManifestFileDescriptor::optional("semanticFamilyAudit"),
    RawExportManifestFileDescriptor::optional("semanticNbtKeyDistribution"),
    RawExportManifestFileDescriptor::optional("semanticIdentityNormalizationReport"),
    RawExportManifestFileDescriptor::optional("neiHandlerAnomalies"),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DistDataSchemaDescriptor {
    pub key: &'static str,
    pub schema_version: &'static str,
}

impl DistDataSchemaDescriptor {
    const fn new(key: &'static str, schema_version: &'static str) -> Self {
        Self {
            key,
            schema_version,
        }
    }
}

pub const DIST_DATA_SCHEMA_DESCRIPTORS: &[DistDataSchemaDescriptor] = &[
    DistDataSchemaDescriptor::new("runtimeManifest", RUST_RUNTIME_MANIFEST_SCHEMA_VERSION),
    DistDataSchemaDescriptor::new(
        "rawExportAbiValidationReport",
        RAW_EXPORT_ABI_VALIDATION_SCHEMA_VERSION,
    ),
    DistDataSchemaDescriptor::new(
        "nativeUiExportAbiValidationReport",
        NATIVE_UI_EXPORT_ABI_VALIDATION_SCHEMA_VERSION,
    ),
    DistDataSchemaDescriptor::new("packValidationReport", PACK_ABI_VALIDATION_SCHEMA_VERSION),
];

pub const UI_PACK_SCHEMA_DESCRIPTORS: &[DistDataSchemaDescriptor] = &[
    DistDataSchemaDescriptor::new("assetsManifest", "neonei/ui-assets-manifest/current"),
    DistDataSchemaDescriptor::new("report", "neonei/ui-pack-report/current"),
    DistDataSchemaDescriptor::new("abiValidationReport", UI_PACK_ABI_VALIDATION_SCHEMA_VERSION),
    DistDataSchemaDescriptor::new(
        "abiValidationReportPath",
        UI_PACK_ABI_VALIDATION_REPORT_PATH,
    ),
    DistDataSchemaDescriptor::new("templatePackSchema", "neonei/ui-template-pack/current"),
    DistDataSchemaDescriptor::new("bindingPackSchema", "neonei/ui-binding-pack/current"),
    DistDataSchemaDescriptor::new("stringPackSchema", "neonei/ui-string-pack/current"),
];

pub fn raw_export_schema_section() -> Value {
    validate_schema_catalog_descriptors();
    json!({
        "schemaVersion": RAW_EXPORT_SCHEMA_VERSION,
        "manifest": {
            "schemaVersion": RAW_EXPORT_MANIFEST_SCHEMA_VERSION,
            "requiredFiles": raw_export_required_manifest_files(),
            "optionalFiles": raw_export_optional_manifest_files(),
            "pathPolicy": RAW_MANIFEST_PATH_POLICY,
            "collections": manifest_collection_catalog()
        },
        "validationReport": {
            "path": "rust/raw-export-abi-validation-report.json",
            "schemaVersion": RAW_EXPORT_ABI_VALIDATION_SCHEMA_VERSION,
            "policy": RAW_EXPORT_VALIDATION_POLICY
        },
        "nativeUiValidationReport": {
            "rawExportPath": NATIVE_UI_VALIDATION_DEFAULT_PATH,
            "compilerReportPath": NATIVE_UI_EXPORT_ABI_VALIDATION_REPORT_PATH,
            "schemaVersion": NATIVE_UI_EXPORT_ABI_VALIDATION_SCHEMA_VERSION,
            "requiredRawSchemaVersion": NESQL_NATIVE_UI_VALIDATION_SCHEMA_VERSION,
            "policy": NATIVE_UI_EXPORT_VALIDATION_POLICY
        },
        "nativeBackground": {
            "requiredSemanticFields": ["status", "kind", "width", "height", "coordinateSpace"],
            "supportedStatus": ["semantic", "captured"],
            "assetRefPolicy": "ignored-if-present",
            "scaling": ["nine-slice", "stretch", "none"],
            "strictPolicy": NATIVE_BACKGROUND_STRICT_POLICY
        },
        "nativeFrame": {
            "status": "retired",
            "required": false,
            "coordinateSpace": "nei_pixels",
            "assetRootPolicy": "retired",
            "retiredArtifacts": ["nei-frame-png"],
            "strictPolicy": NATIVE_FRAME_STRICT_POLICY
        }
    })
}

pub fn dist_data_schema_section() -> Value {
    validate_schema_catalog_descriptors();
    let mut object = Map::new();
    object.insert(
        "schemaVersion".to_string(),
        json!(COMPILED_DIST_SCHEMA_VERSION),
    );
    object.insert("manifest".to_string(), json!("neonei/dist-data/current"));
    insert_schema_descriptors(&mut object, DIST_DATA_SCHEMA_DESCRIPTORS);
    object.insert(
        "runtimeEntrypoints".to_string(),
        json!(runtime_entrypoint_paths()),
    );
    object.insert("runtimeArtifacts".to_string(), runtime_artifact_catalog());
    object.insert("uiPack".to_string(), ui_pack_schema_section());
    object.insert(
        "recipeUiPayloadIndex".to_string(),
        json!("neonei/recipe-ui-payload-index/v1"),
    );
    Value::Object(object)
}

pub fn raw_export_required_manifest_files() -> Vec<&'static str> {
    validate_raw_export_manifest_descriptors();
    RAW_EXPORT_MANIFEST_FILE_DESCRIPTORS
        .iter()
        .filter(|descriptor| descriptor.required)
        .map(|descriptor| descriptor.key)
        .collect()
}

pub fn raw_export_optional_manifest_files() -> Vec<&'static str> {
    validate_raw_export_manifest_descriptors();
    RAW_EXPORT_MANIFEST_FILE_DESCRIPTORS
        .iter()
        .filter(|descriptor| !descriptor.required)
        .map(|descriptor| descriptor.key)
        .collect()
}

pub fn runtime_entrypoint_paths() -> Vec<&'static str> {
    RUST_RUNTIME_ENTRYPOINTS
        .iter()
        .map(|spec| spec.path)
        .collect()
}

pub fn validate_schema_catalog_descriptors() {
    validate_raw_export_manifest_descriptors();
    validate_schema_descriptors("dist-data", DIST_DATA_SCHEMA_DESCRIPTORS);
    validate_schema_descriptors("ui-pack", UI_PACK_SCHEMA_DESCRIPTORS);
}

fn ui_pack_schema_section() -> Value {
    let mut object = Map::new();
    insert_schema_descriptors(&mut object, UI_PACK_SCHEMA_DESCRIPTORS);
    Value::Object(object)
}

fn insert_schema_descriptors(
    object: &mut Map<String, Value>,
    descriptors: &[DistDataSchemaDescriptor],
) {
    validate_schema_descriptors("schema", descriptors);
    for descriptor in descriptors {
        object.insert(descriptor.key.to_string(), json!(descriptor.schema_version));
    }
}

fn validate_raw_export_manifest_descriptors() {
    if RAW_EXPORT_MANIFEST_FILE_DESCRIPTORS.is_empty() {
        panic!("schema catalog raw-export manifest descriptors must not be empty");
    }
    let mut keys = BTreeSet::new();
    let mut required_count = 0usize;
    for descriptor in RAW_EXPORT_MANIFEST_FILE_DESCRIPTORS {
        require_non_empty("raw-export manifest file key", descriptor.key);
        if !keys.insert(descriptor.key) {
            panic!(
                "duplicate schema catalog raw-export manifest file descriptor: {}",
                descriptor.key
            );
        }
        if descriptor.required {
            required_count += 1;
        }
    }
    if required_count == 0 {
        panic!("schema catalog raw-export manifest must declare required files");
    }
}

fn validate_schema_descriptors(label: &str, descriptors: &[DistDataSchemaDescriptor]) {
    if descriptors.is_empty() {
        panic!("schema catalog {label} descriptors must not be empty");
    }
    let mut keys = BTreeSet::new();
    for descriptor in descriptors {
        require_non_empty("schema catalog descriptor key", descriptor.key);
        require_non_empty(
            "schema catalog descriptor schema version",
            descriptor.schema_version,
        );
        if !keys.insert(descriptor.key) {
            panic!(
                "duplicate schema catalog {label} descriptor key: {}",
                descriptor.key
            );
        }
    }
}

fn require_non_empty(label: &str, value: &str) {
    if value.trim().is_empty() {
        panic!("{label} must be non-empty");
    }
}
