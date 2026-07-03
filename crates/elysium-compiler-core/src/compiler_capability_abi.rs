//! Compiler capability ABI catalog.
//!
//! Keep the public command surface, compile scopes, Native UI capability
//! requirements, and fail-fast policy in one module. NeoNEI validates this
//! catalog during the external compiler handshake; schema generation consumes it
//! so the executable's published surface cannot drift from the runtime gate.

use crate::compiler_command_catalog::compiler_command_catalog;
pub use crate::compiler_command_catalog::{
    COMPILER_COMMANDS, REQUIRED_COMPILER_COMMANDS, SCHEMA_HASH_COMMANDS_INPUT,
};
use crate::compiler_scope_catalog::compile_scope_catalog;
pub use crate::compiler_scope_catalog::{COMPILE_SCOPES, SCHEMA_HASH_SCOPES_INPUT};
use crate::native_ui_export_abi_catalog::NATIVE_UI_VALIDATION_DEFAULT_PATH;
use serde_json::{json, Value};

pub const COMPILER_CAPABILITY_ABI_VERSION: &str = "elysium.compiler.capability.v1";

pub const NATIVE_UI_REQUIRED_CAPABILITIES: &[&str] = &[
    "native_ui.surface",
    "native_ui.design_space_coordinates",
    "native_ui.background_asset",
];
pub const NATIVE_UI_REQUIRED_FILES: &[&str] = &[
    "native-ui/families.jsonl.zst",
    "native-ui/surfaces.jsonl.zst",
    "native-ui/slots.bin",
    NATIVE_UI_VALIDATION_DEFAULT_PATH,
];
pub const NATIVE_UI_COORDINATE_SPACE: &str = "nei_pixels";
pub const NATIVE_UI_RUNTIME_TRANSFORM: &str = "uniform-scale-to-fit-only";
pub const NATIVE_UI_FALLBACK_POLICY: &str =
    "missing required native UI capture is a validation error";

pub const COMPILER_POLICY_LEGACY_FALLBACK: &str = "forbidden";
pub const COMPILER_POLICY_MISSING_CAPABILITY: &str = "fail-fast";
pub const COMPILER_POLICY_HOT_PATH_ENCODING: &str = "binary-pack-preferred";
pub const COMPILER_POLICY_FULL_EXPORT_VALIDATION: &str = "milestone-gate-only";

pub const SCHEMA_HASH_COMPILER_CAPABILITY_INPUT: &str =
    "compiler-capability-abi=elysium.compiler.capability.v1";

pub fn compiler_capability_abi_catalog() -> Value {
    json!({
        "name": "elysium.compiler.capability",
        "version": COMPILER_CAPABILITY_ABI_VERSION,
        "commands": COMPILER_COMMANDS,
        "commandCatalog": compiler_command_catalog(),
        "requiredCommands": REQUIRED_COMPILER_COMMANDS,
        "compileScopes": COMPILE_SCOPES,
        "compileScopeCatalog": compile_scope_catalog(),
        "nativeUi": {
            "requiredCapabilities": NATIVE_UI_REQUIRED_CAPABILITIES,
            "requiredFiles": NATIVE_UI_REQUIRED_FILES,
            "coordinateSpace": NATIVE_UI_COORDINATE_SPACE,
            "runtimeTransform": NATIVE_UI_RUNTIME_TRANSFORM,
            "fallbackPolicy": NATIVE_UI_FALLBACK_POLICY
        },
        "policy": {
            "legacyFallback": COMPILER_POLICY_LEGACY_FALLBACK,
            "missingCapability": COMPILER_POLICY_MISSING_CAPABILITY,
            "hotPathEncoding": COMPILER_POLICY_HOT_PATH_ENCODING,
            "fullExportValidation": COMPILER_POLICY_FULL_EXPORT_VALIDATION
        },
        "schemaHashInputs": {
            "compilerCapabilityAbi": SCHEMA_HASH_COMPILER_CAPABILITY_INPUT,
            "commands": SCHEMA_HASH_COMMANDS_INPUT,
            "scopes": SCHEMA_HASH_SCOPES_INPUT
        }
    })
}
