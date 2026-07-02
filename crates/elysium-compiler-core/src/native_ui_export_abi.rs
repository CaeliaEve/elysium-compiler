use crate::io::{sha256_file, write_json_value};
use crate::json_ext::{read_json_file, value_string, value_u64};
use crate::manifest::{portable_relative_path, read_manifest};
use crate::version::EXPORT_ABI_VERSION;
use anyhow::{anyhow, Result};
use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

pub const NATIVE_UI_EXPORT_ABI_VALIDATION_SCHEMA_VERSION: &str =
    "elysium-compiler/native-ui-export-abi-validation/v1";
pub const NESQL_NATIVE_UI_VALIDATION_SCHEMA_VERSION: &str =
    "nesqlpp/raw-export/alpha1/native-ui-validation";
pub const NATIVE_UI_EXPORT_ABI_VALIDATION_REPORT_PATH: &str =
    "rust/native-ui-export-abi-validation-report.json";
pub const NATIVE_UI_VALIDATION_MANIFEST_KEY: &str = "nativeUiValidation";
pub const NATIVE_UI_VALIDATION_DEFAULT_PATH: &str = "validation/native-ui-abi.json";

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeUiExportAbiValidationReport {
    pub schema_version: &'static str,
    pub export_abi_version: &'static str,
    pub generated_at: &'static str,
    pub status: &'static str,
    pub manifest_logical_name: &'static str,
    pub manifest_path: Option<String>,
    pub report_path: Option<String>,
    pub report_bytes: Option<u64>,
    pub report_sha256: Option<String>,
    pub raw_report_schema_version: Option<String>,
    pub raw_report_status: Option<String>,
    pub layout_count: u64,
    pub slot_count: u64,
    pub primitive_count: u64,
    pub missing_surface_count: u64,
    pub slot_bounds_violation_count: u64,
    pub primitive_bounds_violation_count: u64,
    pub background_bounds_violation_count: u64,
    pub coordinate_contract_violation_count: u64,
    pub missing_report: bool,
    pub schema_violations: Vec<String>,
    pub path_violations: Vec<String>,
    pub contract_violations: Vec<String>,
    pub samples: NativeUiExportAbiSamples,
    pub policy: NativeUiExportAbiPolicy,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeUiExportAbiSamples {
    pub missing_surface: Vec<String>,
    pub slot_bounds: Vec<String>,
    pub primitive_bounds: Vec<String>,
    pub background_bounds: Vec<String>,
    pub coordinate_contract: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeUiExportAbiPolicy {
    pub missing_report: &'static str,
    pub schema_mismatch: &'static str,
    pub blocked_raw_report: &'static str,
    pub geometry_contract: &'static str,
    pub path_portability: &'static str,
    pub legacy_fallback: &'static str,
}

impl NativeUiExportAbiValidationReport {
    pub fn is_blocked(&self) -> bool {
        self.missing_report
            || !self.schema_violations.is_empty()
            || !self.path_violations.is_empty()
            || !self.contract_violations.is_empty()
    }
}

pub fn native_ui_export_abi_validation_report_path(output: &Path) -> PathBuf {
    output.join(NATIVE_UI_EXPORT_ABI_VALIDATION_REPORT_PATH)
}

pub fn write_native_ui_export_abi_validation_report(
    input: &Path,
    output: &Path,
    strict: bool,
) -> Result<NativeUiExportAbiValidationReport> {
    let report = validate_native_ui_export_abi(input)?;
    let path = native_ui_export_abi_validation_report_path(output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    write_json_value(&path, &serde_json::to_value(&report)?)?;
    if strict && report.is_blocked() {
        return Err(anyhow!(
            "native UI export ABI validation blocked: missing report={}, schema violations={}, path violations={}, contract violations={}",
            report.missing_report,
            report.schema_violations.len(),
            report.path_violations.len(),
            report.contract_violations.len()
        ));
    }
    Ok(report)
}

pub fn validate_native_ui_export_abi(input: &Path) -> Result<NativeUiExportAbiValidationReport> {
    let manifest = read_manifest(input)?;
    let mut state = NativeUiExportAbiState {
        manifest_path: manifest
            .files
            .get(NATIVE_UI_VALIDATION_MANIFEST_KEY)
            .cloned(),
        ..Default::default()
    };

    let Some(manifest_path_value) = state.manifest_path.clone() else {
        state.missing_report = true;
        state.schema_violations.push(format!(
            "raw export manifest must declare {} -> {}",
            NATIVE_UI_VALIDATION_MANIFEST_KEY, NATIVE_UI_VALIDATION_DEFAULT_PATH
        ));
        return Ok(state.into_report());
    };

    let Some(portable_path) = portable_relative_path(&manifest_path_value) else {
        state.missing_report = true;
        state.path_violations.push(format!(
            "{}:{} violates portable-relative native UI validation path policy",
            NATIVE_UI_VALIDATION_MANIFEST_KEY, manifest_path_value
        ));
        return Ok(state.into_report());
    };

    let normalized = portable_path.to_string_lossy().replace('\\', "/");
    state.report_path = Some(normalized.clone());
    if normalized != NATIVE_UI_VALIDATION_DEFAULT_PATH {
        state.schema_violations.push(format!(
            "{} must point to {} but was {}",
            NATIVE_UI_VALIDATION_MANIFEST_KEY, NATIVE_UI_VALIDATION_DEFAULT_PATH, normalized
        ));
    }

    let path = input.join(&portable_path);
    if !path.exists() {
        state.missing_report = true;
        state.contract_violations.push(format!(
            "native UI validation report file is missing: {}",
            manifest_path_value
        ));
        return Ok(state.into_report());
    }
    if path.is_dir() {
        state.missing_report = true;
        state.path_violations.push(format!(
            "{}:{} points to a directory; native UI validation report must be a JSON file",
            NATIVE_UI_VALIDATION_MANIFEST_KEY, manifest_path_value
        ));
        return Ok(state.into_report());
    }

    let metadata = path.metadata()?;
    state.report_bytes = Some(metadata.len());
    state.report_sha256 = Some(sha256_file(&path)?);
    let raw_report = read_json_file(&path)?;
    state.raw_report_schema_version = value_string(&raw_report, "schemaVersion");
    state.raw_report_status = value_string(&raw_report, "status");
    state.layout_count = value_u64(&raw_report, "layoutCount").unwrap_or(0);
    state.slot_count = value_u64(&raw_report, "slotCount").unwrap_or(0);
    state.primitive_count = value_u64(&raw_report, "primitiveCount").unwrap_or(0);
    state.missing_surface_count = value_u64(&raw_report, "missingSurfaceCount").unwrap_or(0);
    state.slot_bounds_violation_count =
        value_u64(&raw_report, "slotBoundsViolationCount").unwrap_or(0);
    state.primitive_bounds_violation_count =
        value_u64(&raw_report, "primitiveBoundsViolationCount").unwrap_or(0);
    state.background_bounds_violation_count =
        value_u64(&raw_report, "backgroundBoundsViolationCount").unwrap_or(0);
    state.coordinate_contract_violation_count =
        value_u64(&raw_report, "coordinateContractViolationCount").unwrap_or(0);
    state.samples = read_samples(&raw_report);

    if state.raw_report_schema_version.as_deref() != Some(NESQL_NATIVE_UI_VALIDATION_SCHEMA_VERSION)
    {
        state.schema_violations.push(format!(
            "schemaVersion must be {} but was {}",
            NESQL_NATIVE_UI_VALIDATION_SCHEMA_VERSION,
            state
                .raw_report_schema_version
                .as_deref()
                .unwrap_or("<missing>")
        ));
    }
    if state.raw_report_status.as_deref() != Some("ok") {
        state.contract_violations.push(format!(
            "NESQL++ native UI validation status must be ok but was {}",
            state.raw_report_status.as_deref().unwrap_or("<missing>")
        ));
    }
    if state.layout_count == 0 {
        state
            .contract_violations
            .push("layoutCount must be greater than zero".to_string());
    }
    if state.slot_count == 0 {
        state
            .contract_violations
            .push("slotCount must be greater than zero".to_string());
    }
    push_counter_violation(
        &mut state.contract_violations,
        "missingSurfaceCount",
        state.missing_surface_count,
    );
    push_counter_violation(
        &mut state.contract_violations,
        "slotBoundsViolationCount",
        state.slot_bounds_violation_count,
    );
    push_counter_violation(
        &mut state.contract_violations,
        "primitiveBoundsViolationCount",
        state.primitive_bounds_violation_count,
    );
    push_counter_violation(
        &mut state.contract_violations,
        "backgroundBoundsViolationCount",
        state.background_bounds_violation_count,
    );
    push_counter_violation(
        &mut state.contract_violations,
        "coordinateContractViolationCount",
        state.coordinate_contract_violation_count,
    );

    Ok(state.into_report())
}

#[derive(Default)]
struct NativeUiExportAbiState {
    manifest_path: Option<String>,
    report_path: Option<String>,
    report_bytes: Option<u64>,
    report_sha256: Option<String>,
    raw_report_schema_version: Option<String>,
    raw_report_status: Option<String>,
    layout_count: u64,
    slot_count: u64,
    primitive_count: u64,
    missing_surface_count: u64,
    slot_bounds_violation_count: u64,
    primitive_bounds_violation_count: u64,
    background_bounds_violation_count: u64,
    coordinate_contract_violation_count: u64,
    missing_report: bool,
    schema_violations: Vec<String>,
    path_violations: Vec<String>,
    contract_violations: Vec<String>,
    samples: NativeUiExportAbiSamples,
}

impl NativeUiExportAbiState {
    fn into_report(self) -> NativeUiExportAbiValidationReport {
        let status = if self.missing_report
            || !self.schema_violations.is_empty()
            || !self.path_violations.is_empty()
            || !self.contract_violations.is_empty()
        {
            "blocked"
        } else {
            "ok"
        };
        NativeUiExportAbiValidationReport {
            schema_version: NATIVE_UI_EXPORT_ABI_VALIDATION_SCHEMA_VERSION,
            export_abi_version: EXPORT_ABI_VERSION,
            generated_at: "deterministic-rust-compiler",
            status,
            manifest_logical_name: NATIVE_UI_VALIDATION_MANIFEST_KEY,
            manifest_path: self.manifest_path,
            report_path: self.report_path,
            report_bytes: self.report_bytes,
            report_sha256: self.report_sha256,
            raw_report_schema_version: self.raw_report_schema_version,
            raw_report_status: self.raw_report_status,
            layout_count: self.layout_count,
            slot_count: self.slot_count,
            primitive_count: self.primitive_count,
            missing_surface_count: self.missing_surface_count,
            slot_bounds_violation_count: self.slot_bounds_violation_count,
            primitive_bounds_violation_count: self.primitive_bounds_violation_count,
            background_bounds_violation_count: self.background_bounds_violation_count,
            coordinate_contract_violation_count: self.coordinate_contract_violation_count,
            missing_report: self.missing_report,
            schema_violations: self.schema_violations,
            path_violations: self.path_violations,
            contract_violations: self.contract_violations,
            samples: self.samples,
            policy: NativeUiExportAbiPolicy {
                missing_report: "fail-closed",
                schema_mismatch: "fail-closed",
                blocked_raw_report: "fail-closed",
                geometry_contract:
                    "layouts, slots, and rect primitives must be bounded, surface-complete, and use NEI pixel coordinates with uniform scaling",
                path_portability:
                    "portable-relative raw-export path only; expected validation/native-ui-abi.json",
                legacy_fallback: "forbidden",
            },
        }
    }
}

fn read_samples(raw_report: &Value) -> NativeUiExportAbiSamples {
    NativeUiExportAbiSamples {
        missing_surface: read_string_array(raw_report, "missingSurfaceSamples"),
        slot_bounds: read_string_array(raw_report, "slotBoundsSamples"),
        primitive_bounds: read_string_array(raw_report, "primitiveBoundsSamples"),
        background_bounds: read_string_array(raw_report, "backgroundBoundsSamples"),
        coordinate_contract: read_string_array(raw_report, "coordinateContractSamples"),
    }
}

fn read_string_array(value: &Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(|entries| {
            entries
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn push_counter_violation(violations: &mut Vec<String>, key: &str, value: u64) {
    if value > 0 {
        violations.push(format!("{key} must be zero but was {value}"));
    }
}
