use crate::abi::abi_catalog;
use crate::compiler_capability_abi::{COMPILER_COMMANDS, COMPILE_SCOPES};
use crate::compiler_command_catalog::compiler_command_catalog;
use crate::io::write_json_value;
use crate::schema_catalog::{
    dist_data_schema_section, raw_export_schema_section, SCHEMA_CATALOG_SCHEMA_VERSION,
};
use crate::stages::compile_kernel_catalog;
use crate::version::metadata as compiler_metadata;
use anyhow::Result;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

pub fn schema_catalog() -> Value {
    json!({
        "schemaVersion": SCHEMA_CATALOG_SCHEMA_VERSION,
        "abi": abi_catalog(),
        "compiler": {
            "name": "elysium-compiler",
            "currentCrate": "elysium-compiler-core",
            "metadata": compiler_metadata(),
            "cli": {
                "commands": COMPILER_COMMANDS,
                "compileScopes": COMPILE_SCOPES,
                "commandCatalog": compiler_command_catalog()
            },
            "compileKernel": compile_kernel_catalog()
        },
        "rawExport": raw_export_schema_section(),
        "distData": dist_data_schema_section()
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
