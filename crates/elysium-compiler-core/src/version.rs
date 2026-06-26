pub const COMPILER_NAME: &str = "elysium-compiler";
pub const COMPILER_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const RAW_EXPORT_SCHEMA_VERSION: &str = "1.0";
pub const COMPILED_DIST_SCHEMA_VERSION: &str = "1.0";
pub const SCHEMA_CATALOG_VERSION: &str = "elysium-compiler/schema-catalog/v1";

pub fn schema_hash() -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    for part in [
        COMPILER_NAME,
        COMPILER_VERSION,
        SCHEMA_CATALOG_VERSION,
        RAW_EXPORT_SCHEMA_VERSION,
        COMPILED_DIST_SCHEMA_VERSION,
        "commands=compile,inspect,validate,schemas",
        "scopes=all,native-ui,search,browser,recipes,ui,textures",
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
        "schemaHash": schema_hash(),
    })
}
