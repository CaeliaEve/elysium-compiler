use crate::io::write_json_value;
use crate::json_ext::{value_string, value_u64};
use crate::manifest::portable_relative_path;
use crate::native_ui_export_abi_catalog::*;
use crate::session::{RawExportSession, RelativeInputKind};
use crate::version::EXPORT_ABI_VERSION;
use anyhow::{anyhow, Result};
use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

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
    pub rect_count: u64,
    pub primitive_count: u64,
    pub missing_surface_count: u64,
    pub slot_bounds_violation_count: u64,
    pub rect_bounds_violation_count: u64,
    pub primitive_bounds_violation_count: u64,
    pub background_bounds_violation_count: u64,
    pub coordinate_contract_violation_count: u64,
    pub interaction_contract_violation_count: u64,
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
    pub rect_bounds: Vec<String>,
    pub primitive_bounds: Vec<String>,
    pub background_bounds: Vec<String>,
    pub coordinate_contract: Vec<String>,
    pub interaction_contract: Vec<String>,
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

pub(crate) fn write_native_ui_export_abi_validation_report_with_session(
    session: &RawExportSession,
    output: &Path,
    strict: bool,
) -> Result<NativeUiExportAbiValidationReport> {
    let report = validate_native_ui_export_abi_with_session(session)?;
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

#[cfg(test)]
pub fn validate_native_ui_export_abi(input: &Path) -> Result<NativeUiExportAbiValidationReport> {
    let session = RawExportSession::open(input)?;
    validate_native_ui_export_abi_with_session(&session)
}

pub(crate) fn validate_native_ui_export_abi_with_session(
    session: &RawExportSession,
) -> Result<NativeUiExportAbiValidationReport> {
    Ok(session
        .native_ui_export_abi_validation(|| validate_native_ui_export_abi_uncached(session))?
        .as_ref()
        .clone())
}

fn validate_native_ui_export_abi_uncached(
    session: &RawExportSession,
) -> Result<NativeUiExportAbiValidationReport> {
    let manifest = session.manifest();
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

    match session.relative_input_kind(&manifest_path_value)? {
        RelativeInputKind::Missing => {
            state.missing_report = true;
            state.contract_violations.push(format!(
                "native UI validation report file is missing: {}",
                manifest_path_value
            ));
            return Ok(state.into_report());
        }
        RelativeInputKind::Directory | RelativeInputKind::Unsupported => {
            state.missing_report = true;
            state.path_violations.push(format!(
                "{}:{} points to a non-file; native UI validation report must be a JSON file",
                NATIVE_UI_VALIDATION_MANIFEST_KEY, manifest_path_value
            ));
            return Ok(state.into_report());
        }
        RelativeInputKind::File => {}
    }

    let descriptor = session
        .describe_relative_file(&manifest_path_value)?
        .ok_or_else(|| anyhow!("native UI validation report disappeared during session"))?;
    state.report_bytes = Some(descriptor.bytes);
    state.report_sha256 = Some(descriptor.sha256.clone());
    let raw_report = session
        .read_manifest_json(NATIVE_UI_VALIDATION_MANIFEST_KEY)?
        .ok_or_else(|| anyhow!("native UI validation report is not declared"))?;
    state.raw_report_schema_version = value_string(raw_report.as_ref(), "schemaVersion");
    state.raw_report_status = value_string(raw_report.as_ref(), "status");
    state.layout_count =
        value_u64(raw_report.as_ref(), NATIVE_UI_EXPORT_LAYOUT_COUNT_FIELD).unwrap_or(0);
    state.slot_count =
        value_u64(raw_report.as_ref(), NATIVE_UI_EXPORT_SLOT_COUNT_FIELD).unwrap_or(0);
    state.rect_count =
        value_u64(raw_report.as_ref(), NATIVE_UI_EXPORT_RECT_COUNT_FIELD).unwrap_or(0);
    state.primitive_count =
        value_u64(&raw_report, NATIVE_UI_EXPORT_PRIMITIVE_COUNT_FIELD).unwrap_or(0);
    state.missing_surface_count =
        value_u64(&raw_report, NATIVE_UI_EXPORT_MISSING_SURFACE_COUNT_FIELD).unwrap_or(0);
    state.slot_bounds_violation_count = value_u64(
        &raw_report,
        NATIVE_UI_EXPORT_SLOT_BOUNDS_VIOLATION_COUNT_FIELD,
    )
    .unwrap_or(0);
    state.rect_bounds_violation_count = value_u64(
        &raw_report,
        NATIVE_UI_EXPORT_RECT_BOUNDS_VIOLATION_COUNT_FIELD,
    )
    .unwrap_or(0);
    state.primitive_bounds_violation_count = value_u64(
        &raw_report,
        NATIVE_UI_EXPORT_PRIMITIVE_BOUNDS_VIOLATION_COUNT_FIELD,
    )
    .unwrap_or(0);
    state.background_bounds_violation_count = value_u64(
        &raw_report,
        NATIVE_UI_EXPORT_BACKGROUND_BOUNDS_VIOLATION_COUNT_FIELD,
    )
    .unwrap_or(0);
    state.coordinate_contract_violation_count = value_u64(
        &raw_report,
        NATIVE_UI_EXPORT_COORDINATE_CONTRACT_VIOLATION_COUNT_FIELD,
    )
    .unwrap_or(0);
    state.interaction_contract_violation_count = value_u64(
        &raw_report,
        NATIVE_UI_EXPORT_INTERACTION_CONTRACT_VIOLATION_COUNT_FIELD,
    )
    .unwrap_or(0);
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
    if state.raw_report_status.as_deref() != Some(NATIVE_UI_EXPORT_STATUS_OK) {
        state.contract_violations.push(format!(
            "NESQL++ native UI validation status must be ok but was {}",
            state.raw_report_status.as_deref().unwrap_or("<missing>")
        ));
    }
    for key in NATIVE_UI_EXPORT_REQUIRED_POSITIVE_COUNTERS {
        let value = state.counter_value(key);
        push_positive_counter_requirement(&mut state.contract_violations, key, value);
    }
    for key in NATIVE_UI_EXPORT_ZERO_VIOLATION_COUNTERS {
        let value = state.counter_value(key);
        push_counter_violation(&mut state.contract_violations, key, value);
    }

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
    rect_count: u64,
    primitive_count: u64,
    missing_surface_count: u64,
    slot_bounds_violation_count: u64,
    rect_bounds_violation_count: u64,
    primitive_bounds_violation_count: u64,
    background_bounds_violation_count: u64,
    coordinate_contract_violation_count: u64,
    interaction_contract_violation_count: u64,
    missing_report: bool,
    schema_violations: Vec<String>,
    path_violations: Vec<String>,
    contract_violations: Vec<String>,
    samples: NativeUiExportAbiSamples,
}

impl NativeUiExportAbiState {
    fn counter_value(&self, key: &str) -> u64 {
        match key {
            NATIVE_UI_EXPORT_LAYOUT_COUNT_FIELD => self.layout_count,
            NATIVE_UI_EXPORT_SLOT_COUNT_FIELD => self.slot_count,
            NATIVE_UI_EXPORT_RECT_COUNT_FIELD => self.rect_count,
            NATIVE_UI_EXPORT_PRIMITIVE_COUNT_FIELD => self.primitive_count,
            NATIVE_UI_EXPORT_MISSING_SURFACE_COUNT_FIELD => self.missing_surface_count,
            NATIVE_UI_EXPORT_SLOT_BOUNDS_VIOLATION_COUNT_FIELD => self.slot_bounds_violation_count,
            NATIVE_UI_EXPORT_RECT_BOUNDS_VIOLATION_COUNT_FIELD => self.rect_bounds_violation_count,
            NATIVE_UI_EXPORT_PRIMITIVE_BOUNDS_VIOLATION_COUNT_FIELD => {
                self.primitive_bounds_violation_count
            }
            NATIVE_UI_EXPORT_BACKGROUND_BOUNDS_VIOLATION_COUNT_FIELD => {
                self.background_bounds_violation_count
            }
            NATIVE_UI_EXPORT_COORDINATE_CONTRACT_VIOLATION_COUNT_FIELD => {
                self.coordinate_contract_violation_count
            }
            NATIVE_UI_EXPORT_INTERACTION_CONTRACT_VIOLATION_COUNT_FIELD => {
                self.interaction_contract_violation_count
            }
            _ => 0,
        }
    }

    fn into_report(self) -> NativeUiExportAbiValidationReport {
        let status = if self.missing_report
            || !self.schema_violations.is_empty()
            || !self.path_violations.is_empty()
            || !self.contract_violations.is_empty()
        {
            NATIVE_UI_EXPORT_STATUS_BLOCKED
        } else {
            NATIVE_UI_EXPORT_STATUS_OK
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
            rect_count: self.rect_count,
            primitive_count: self.primitive_count,
            missing_surface_count: self.missing_surface_count,
            slot_bounds_violation_count: self.slot_bounds_violation_count,
            rect_bounds_violation_count: self.rect_bounds_violation_count,
            primitive_bounds_violation_count: self.primitive_bounds_violation_count,
            background_bounds_violation_count: self.background_bounds_violation_count,
            coordinate_contract_violation_count: self.coordinate_contract_violation_count,
            interaction_contract_violation_count: self.interaction_contract_violation_count,
            missing_report: self.missing_report,
            schema_violations: self.schema_violations,
            path_violations: self.path_violations,
            contract_violations: self.contract_violations,
            samples: self.samples,
            policy: NativeUiExportAbiPolicy {
                missing_report: NATIVE_UI_EXPORT_POLICY_FAIL_CLOSED,
                schema_mismatch: NATIVE_UI_EXPORT_POLICY_FAIL_CLOSED,
                blocked_raw_report: NATIVE_UI_EXPORT_POLICY_FAIL_CLOSED,
                geometry_contract: NATIVE_UI_EXPORT_POLICY_GEOMETRY_CONTRACT,
                path_portability: NATIVE_UI_EXPORT_POLICY_PATH_PORTABILITY,
                legacy_fallback: NATIVE_UI_EXPORT_POLICY_LEGACY_FORBIDDEN,
            },
        }
    }
}

fn read_samples(raw_report: &Value) -> NativeUiExportAbiSamples {
    NativeUiExportAbiSamples {
        missing_surface: read_string_array(raw_report, NATIVE_UI_EXPORT_SAMPLE_FIELDS[0]),
        slot_bounds: read_string_array(raw_report, NATIVE_UI_EXPORT_SAMPLE_FIELDS[1]),
        rect_bounds: read_string_array(raw_report, NATIVE_UI_EXPORT_SAMPLE_FIELDS[2]),
        primitive_bounds: read_string_array(raw_report, NATIVE_UI_EXPORT_SAMPLE_FIELDS[3]),
        background_bounds: read_string_array(raw_report, NATIVE_UI_EXPORT_SAMPLE_FIELDS[4]),
        coordinate_contract: read_string_array(raw_report, NATIVE_UI_EXPORT_SAMPLE_FIELDS[5]),
        interaction_contract: read_string_array(raw_report, NATIVE_UI_EXPORT_SAMPLE_FIELDS[6]),
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

fn push_positive_counter_requirement(violations: &mut Vec<String>, key: &str, value: u64) {
    if value == 0 {
        violations.push(format!("{key} must be greater than zero"));
    }
}
