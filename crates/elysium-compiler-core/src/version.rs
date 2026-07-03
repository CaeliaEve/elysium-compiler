use crate::compiler_capability_abi::{
    SCHEMA_HASH_COMMANDS_INPUT, SCHEMA_HASH_COMPILER_CAPABILITY_INPUT, SCHEMA_HASH_SCOPES_INPUT,
};
use crate::manifest::SCHEMA_HASH_MANIFEST_COLLECTION_INPUT;
use crate::native_ui_export_abi_catalog::NATIVE_UI_EXPORT_ABI_VALIDATION_SCHEMA_HASH_INPUT;
use crate::pack_abi::SCHEMA_HASH_RUNTIME_ARTIFACT_CATALOG_INPUT;
use crate::runtime_manifest_abi::{
    SCHEMA_HASH_RUNTIME_MANIFEST_INPUT, SCHEMA_HASH_RUNTIME_REPORT_PAYLOAD_POLICY_INPUT,
};

pub const COMPILER_NAME: &str = "elysium-compiler";
pub const COMPILER_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const RAW_EXPORT_SCHEMA_VERSION: &str = "1.0";
pub const COMPILED_DIST_SCHEMA_VERSION: &str = "1.0";
pub const SCHEMA_CATALOG_VERSION: &str = "elysium-compiler/schema-catalog/v1";
pub const EXPORT_ABI_VERSION: &str = "elysium.export.v2";
pub const PACK_ABI_VERSION: &str = "elysium.pack.v1";
pub const RUNTIME_ABI_VERSION: &str = "elysium.runtime.v1";

pub fn schema_hash() -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    for part in [
        COMPILER_NAME,
        COMPILER_VERSION,
        SCHEMA_CATALOG_VERSION,
        RAW_EXPORT_SCHEMA_VERSION,
        COMPILED_DIST_SCHEMA_VERSION,
        EXPORT_ABI_VERSION,
        PACK_ABI_VERSION,
        RUNTIME_ABI_VERSION,
        SCHEMA_HASH_COMMANDS_INPUT,
        SCHEMA_HASH_SCOPES_INPUT,
        SCHEMA_HASH_COMPILER_CAPABILITY_INPUT,
        SCHEMA_HASH_RUNTIME_MANIFEST_INPUT,
        SCHEMA_HASH_RUNTIME_REPORT_PAYLOAD_POLICY_INPUT,
        SCHEMA_HASH_RUNTIME_ARTIFACT_CATALOG_INPUT,
        SCHEMA_HASH_MANIFEST_COLLECTION_INPUT,
        "raw-export-abi-validation=elysium-compiler/raw-export-abi-validation/v1",
        NATIVE_UI_EXPORT_ABI_VALIDATION_SCHEMA_HASH_INPUT,
        "ui-pack-abi-validation=elysium-compiler/ui-pack-abi-validation/v1",
        "pack-abi-validation=elysium-compiler/pack-abi-validation/v1",
    ] {
        hasher.update(part.as_bytes());
        hasher.update(b"\0");
    }
    format!("sha256:{:x}", hasher.finalize())
}

pub fn metadata() -> serde_json::Value {
    serde_json::json!({
        "name": COMPILER_NAME,
        "version": COMPILER_VERSION,
        "rawExportSchemaVersion": RAW_EXPORT_SCHEMA_VERSION,
        "compiledDistSchemaVersion": COMPILED_DIST_SCHEMA_VERSION,
        "exportAbiVersion": EXPORT_ABI_VERSION,
        "packAbiVersion": PACK_ABI_VERSION,
        "runtimeAbiVersion": RUNTIME_ABI_VERSION,
        "schemaHash": schema_hash(),
    })
}
