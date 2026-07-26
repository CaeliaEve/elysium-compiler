use crate::io::write_json_value;
use crate::manifest::portable_relative_path;
use crate::session::{RawExportSession, RelativeInputKind};
use crate::version::{EXPORT_ABI_VERSION, RAW_EXPORT_SCHEMA_VERSION};
use anyhow::{anyhow, Result};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

pub const RAW_EXPORT_ABI_VALIDATION_SCHEMA_VERSION: &str =
    "elysium-compiler/raw-export-abi-validation/v1";
pub const RAW_EXPORT_ABI_VALIDATION_REPORT_PATH: &str =
    "rust/raw-export-abi-validation-report.json";
pub const REQUIRED_RAW_EXPORT_FILES: &[&str] =
    &["items", "fluids", "recipeIndex", "browserAtlasIndex"];

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RawExportAbiValidationReport {
    pub schema_version: &'static str,
    pub export_abi_version: &'static str,
    pub legacy_raw_export_schema_version: &'static str,
    pub generated_at: &'static str,
    pub status: &'static str,
    pub declared_file_count: usize,
    pub present_file_count: usize,
    pub missing_required_files: Vec<String>,
    pub missing_declared_files: Vec<String>,
    pub path_violations: Vec<String>,
    pub files: Vec<RawExportAbiFileRecord>,
    pub policy: RawExportAbiPolicy,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RawExportAbiFileRecord {
    pub logical_name: String,
    pub manifest_path: String,
    pub path: Option<String>,
    pub required: bool,
    pub status: &'static str,
    pub kind: Option<&'static str>,
    pub bytes: Option<u64>,
    pub sha256: Option<String>,
    pub row_count: Option<u64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RawExportAbiPolicy {
    pub missing_required_file: &'static str,
    pub missing_declared_file: &'static str,
    pub path_portability: &'static str,
    pub legacy_fallback: &'static str,
}

impl RawExportAbiValidationReport {
    pub fn is_blocked(&self) -> bool {
        !self.missing_required_files.is_empty()
            || !self.missing_declared_files.is_empty()
            || !self.path_violations.is_empty()
    }
}

pub fn raw_export_abi_validation_report_path(output: &Path) -> PathBuf {
    output.join(RAW_EXPORT_ABI_VALIDATION_REPORT_PATH)
}

pub(crate) fn write_raw_export_abi_validation_report_with_session(
    session: &RawExportSession,
    output: &Path,
    strict: bool,
) -> Result<RawExportAbiValidationReport> {
    let report = validate_raw_export_abi_with_session(session)?;
    let path = raw_export_abi_validation_report_path(output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    write_json_value(&path, &serde_json::to_value(&report)?)?;
    if strict && report.is_blocked() {
        return Err(anyhow!(
            "raw export ABI validation blocked: missing required={}, missing declared={}, path violations={}",
            report.missing_required_files.len(),
            report.missing_declared_files.len(),
            report.path_violations.len()
        ));
    }
    Ok(report)
}

#[cfg(test)]
pub fn validate_raw_export_abi(input: &Path) -> Result<RawExportAbiValidationReport> {
    let session = RawExportSession::open(input)?;
    validate_raw_export_abi_with_session(&session)
}

pub(crate) fn validate_raw_export_abi_with_session(
    session: &RawExportSession,
) -> Result<RawExportAbiValidationReport> {
    Ok(session
        .raw_export_abi_validation(|| validate_raw_export_abi_uncached(session))?
        .as_ref()
        .clone())
}

fn validate_raw_export_abi_uncached(
    session: &RawExportSession,
) -> Result<RawExportAbiValidationReport> {
    let manifest = session.manifest();
    let mut files = Vec::with_capacity(manifest.files.len());
    let mut missing_declared_files = Vec::new();
    let mut path_violations = Vec::new();
    let mut present_file_count = 0usize;

    for (logical_name, manifest_path) in &manifest.files {
        let required = REQUIRED_RAW_EXPORT_FILES.contains(&logical_name.as_str());
        let Some(portable_path) = portable_relative_path(manifest_path) else {
            path_violations.push(format!(
                "{}:{} violates portable-relative raw export path policy",
                logical_name, manifest_path
            ));
            files.push(RawExportAbiFileRecord {
                logical_name: logical_name.clone(),
                manifest_path: manifest_path.clone(),
                path: None,
                required,
                status: "path-violation",
                kind: None,
                bytes: None,
                sha256: None,
                row_count: None,
            });
            continue;
        };

        let normalized = portable_path.to_string_lossy().replace('\\', "/");
        match session.relative_input_kind(manifest_path)? {
            RelativeInputKind::Missing => {
                missing_declared_files.push(format!("{}:{}", logical_name, normalized));
                files.push(RawExportAbiFileRecord {
                    logical_name: logical_name.clone(),
                    manifest_path: manifest_path.clone(),
                    path: Some(normalized),
                    required,
                    status: "missing",
                    kind: None,
                    bytes: None,
                    sha256: None,
                    row_count: None,
                });
                continue;
            }
            RelativeInputKind::Directory => {
                path_violations.push(format!(
                    "{}:{} points to a non-file; raw export manifest entries must be files",
                    logical_name, normalized
                ));
                files.push(RawExportAbiFileRecord {
                    logical_name: logical_name.clone(),
                    manifest_path: manifest_path.clone(),
                    path: Some(normalized),
                    required,
                    status: "directory",
                    kind: Some("directory"),
                    bytes: None,
                    sha256: None,
                    row_count: None,
                });
                continue;
            }
            RelativeInputKind::Unsupported => {
                path_violations.push(format!(
                    "{}:{} points to an unsupported filesystem entry; raw export manifest entries must be regular files",
                    logical_name, normalized
                ));
                files.push(RawExportAbiFileRecord {
                    logical_name: logical_name.clone(),
                    manifest_path: manifest_path.clone(),
                    path: Some(normalized),
                    required,
                    status: "unsupported",
                    kind: Some("unsupported"),
                    bytes: None,
                    sha256: None,
                    row_count: None,
                });
                continue;
            }
            RelativeInputKind::File => {}
        }

        let descriptor = session
            .describe_relative_file(manifest_path)?
            .ok_or_else(|| {
                anyhow!(
                    "raw-export file disappeared during ABI validation: {}",
                    normalized
                )
            })?;
        present_file_count += 1;
        files.push(RawExportAbiFileRecord {
            logical_name: logical_name.clone(),
            manifest_path: manifest_path.clone(),
            path: Some(normalized),
            required,
            status: "present",
            kind: Some("file"),
            bytes: Some(descriptor.bytes),
            sha256: Some(descriptor.sha256.clone()),
            row_count: descriptor.row_count,
        });
    }

    let missing_required_files = REQUIRED_RAW_EXPORT_FILES
        .iter()
        .filter(|logical_name| !manifest.files.contains_key(**logical_name))
        .map(|logical_name| (*logical_name).to_string())
        .chain(
            files
                .iter()
                .filter(|file| file.required && file.status != "present")
                .map(|file| file.logical_name.clone()),
        )
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    let status = if missing_required_files.is_empty()
        && missing_declared_files.is_empty()
        && path_violations.is_empty()
    {
        "ok"
    } else {
        "blocked"
    };

    Ok(RawExportAbiValidationReport {
        schema_version: RAW_EXPORT_ABI_VALIDATION_SCHEMA_VERSION,
        export_abi_version: EXPORT_ABI_VERSION,
        legacy_raw_export_schema_version: RAW_EXPORT_SCHEMA_VERSION,
        generated_at: "deterministic-rust-compiler",
        status,
        declared_file_count: manifest.files.len(),
        present_file_count,
        missing_required_files,
        missing_declared_files,
        path_violations,
        files,
        policy: RawExportAbiPolicy {
            missing_required_file: "fail-closed",
            missing_declared_file: "fail-closed",
            path_portability:
                "portable-relative-raw-export-paths-only; no drive letters, UNC paths, file URLs, absolute paths, '.', or '..' segments",
            legacy_fallback: "forbidden",
        },
    })
}
