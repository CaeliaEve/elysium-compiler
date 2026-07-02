//! Native UI export ABI catalog.
//!
//! This module is the single source of truth for the NESQL++ native UI
//! validation report consumed by elysium-compiler and by NeoNEI runtime gates.
//! Keep schema versions, report paths, field names, and fail-closed policies
//! together so ABI changes are explicit instead of scattered through validators,
//! manifests, and tests.

use serde_json::{json, Value};

pub const NATIVE_UI_EXPORT_ABI_VALIDATION_SCHEMA_VERSION: &str =
    "elysium-compiler/native-ui-export-abi-validation/v1";
pub const NATIVE_UI_EXPORT_ABI_VALIDATION_SCHEMA_HASH_INPUT: &str =
    "native-ui-export-abi-validation=elysium-compiler/native-ui-export-abi-validation/v1";
pub const NESQL_NATIVE_UI_VALIDATION_SCHEMA_VERSION: &str =
    "nesqlpp/raw-export/alpha1/native-ui-validation";
pub const NATIVE_UI_EXPORT_ABI_VALIDATION_REPORT_PATH: &str =
    "rust/native-ui-export-abi-validation-report.json";
pub const NATIVE_UI_VALIDATION_MANIFEST_KEY: &str = "nativeUiValidation";
pub const NATIVE_UI_VALIDATION_DEFAULT_PATH: &str = "validation/native-ui-abi.json";

pub const NATIVE_UI_EXPORT_STATUS_OK: &str = "ok";
pub const NATIVE_UI_EXPORT_STATUS_BLOCKED: &str = "blocked";
pub const NATIVE_UI_EXPORT_POLICY_FAIL_CLOSED: &str = "fail-closed";
pub const NATIVE_UI_EXPORT_POLICY_LEGACY_FORBIDDEN: &str = "forbidden";
pub const NATIVE_UI_EXPORT_POLICY_GEOMETRY_CONTRACT: &str =
    "layouts, slots, and rect primitives must be bounded, surface-complete, and use NEI pixel coordinates with uniform scaling";
pub const NATIVE_UI_EXPORT_POLICY_PATH_PORTABILITY: &str =
    "portable-relative raw-export path only; expected validation/native-ui-abi.json";

pub const NATIVE_UI_EXPORT_LAYOUT_COUNT_FIELD: &str = "layoutCount";
pub const NATIVE_UI_EXPORT_SLOT_COUNT_FIELD: &str = "slotCount";
pub const NATIVE_UI_EXPORT_RECT_COUNT_FIELD: &str = "rectCount";
pub const NATIVE_UI_EXPORT_PRIMITIVE_COUNT_FIELD: &str = "primitiveCount";
pub const NATIVE_UI_EXPORT_MISSING_SURFACE_COUNT_FIELD: &str = "missingSurfaceCount";
pub const NATIVE_UI_EXPORT_SLOT_BOUNDS_VIOLATION_COUNT_FIELD: &str = "slotBoundsViolationCount";
pub const NATIVE_UI_EXPORT_RECT_BOUNDS_VIOLATION_COUNT_FIELD: &str = "rectBoundsViolationCount";
pub const NATIVE_UI_EXPORT_PRIMITIVE_BOUNDS_VIOLATION_COUNT_FIELD: &str =
    "primitiveBoundsViolationCount";
pub const NATIVE_UI_EXPORT_BACKGROUND_BOUNDS_VIOLATION_COUNT_FIELD: &str =
    "backgroundBoundsViolationCount";
pub const NATIVE_UI_EXPORT_COORDINATE_CONTRACT_VIOLATION_COUNT_FIELD: &str =
    "coordinateContractViolationCount";
pub const NATIVE_UI_EXPORT_INTERACTION_CONTRACT_VIOLATION_COUNT_FIELD: &str =
    "interactionContractViolationCount";

pub const NATIVE_UI_EXPORT_REQUIRED_POSITIVE_COUNTERS: &[&str] = &[
    NATIVE_UI_EXPORT_LAYOUT_COUNT_FIELD,
    NATIVE_UI_EXPORT_SLOT_COUNT_FIELD,
];

pub const NATIVE_UI_EXPORT_ZERO_VIOLATION_COUNTERS: &[&str] = &[
    NATIVE_UI_EXPORT_MISSING_SURFACE_COUNT_FIELD,
    NATIVE_UI_EXPORT_SLOT_BOUNDS_VIOLATION_COUNT_FIELD,
    NATIVE_UI_EXPORT_RECT_BOUNDS_VIOLATION_COUNT_FIELD,
    NATIVE_UI_EXPORT_PRIMITIVE_BOUNDS_VIOLATION_COUNT_FIELD,
    NATIVE_UI_EXPORT_BACKGROUND_BOUNDS_VIOLATION_COUNT_FIELD,
    NATIVE_UI_EXPORT_COORDINATE_CONTRACT_VIOLATION_COUNT_FIELD,
    NATIVE_UI_EXPORT_INTERACTION_CONTRACT_VIOLATION_COUNT_FIELD,
];

pub const NATIVE_UI_EXPORT_SAMPLE_FIELDS: &[&str] = &[
    "missingSurfaceSamples",
    "slotBoundsSamples",
    "rectBoundsSamples",
    "primitiveBoundsSamples",
    "backgroundBoundsSamples",
    "coordinateContractSamples",
    "interactionContractSamples",
];

/// Public Native UI export ABI catalog emitted through the compiler schema ABI.
pub fn native_ui_export_abi_catalog() -> Value {
    json!({
        "validationReport": {
            "path": NATIVE_UI_EXPORT_ABI_VALIDATION_REPORT_PATH,
            "schemaVersion": NATIVE_UI_EXPORT_ABI_VALIDATION_SCHEMA_VERSION,
            "rawExportPath": NATIVE_UI_VALIDATION_DEFAULT_PATH,
            "rawSchemaVersion": NESQL_NATIVE_UI_VALIDATION_SCHEMA_VERSION,
            "manifestKey": NATIVE_UI_VALIDATION_MANIFEST_KEY
        },
        "requiredPositiveCounters": NATIVE_UI_EXPORT_REQUIRED_POSITIVE_COUNTERS,
        "zeroViolationCounters": NATIVE_UI_EXPORT_ZERO_VIOLATION_COUNTERS,
        "sampleFields": NATIVE_UI_EXPORT_SAMPLE_FIELDS,
        "policy": {
            "missingReport": NATIVE_UI_EXPORT_POLICY_FAIL_CLOSED,
            "schemaMismatch": NATIVE_UI_EXPORT_POLICY_FAIL_CLOSED,
            "blockedRawReport": NATIVE_UI_EXPORT_POLICY_FAIL_CLOSED,
            "geometryContract": NATIVE_UI_EXPORT_POLICY_GEOMETRY_CONTRACT,
            "pathPortability": NATIVE_UI_EXPORT_POLICY_PATH_PORTABILITY,
            "legacyFallback": NATIVE_UI_EXPORT_POLICY_LEGACY_FORBIDDEN
        }
    })
}
