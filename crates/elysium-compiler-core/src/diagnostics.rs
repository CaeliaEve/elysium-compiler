use crate::io::normalize_path;
use crate::native_ui_export_abi::{
    validate_native_ui_export_abi_with_session,
    write_native_ui_export_abi_validation_report_with_session,
};
use crate::raw_export::summarize_raw_export;
use crate::raw_export_abi::{
    validate_raw_export_abi_with_session, write_raw_export_abi_validation_report_with_session,
};
use crate::reports::{summarize_runtime_output, write_report, CompilerReport};
use crate::session::RawExportSession;
use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;
use std::time::Instant;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticsMode {
    Inspect,
    Validate,
    Compile,
}

impl DiagnosticsMode {
    fn as_str(self) -> &'static str {
        match self {
            DiagnosticsMode::Inspect => "inspect",
            DiagnosticsMode::Validate => "validate",
            DiagnosticsMode::Compile => "compile",
        }
    }
}

pub fn run_diagnostics_report(
    input: &Path,
    output: Option<&Path>,
    report: &Path,
    strict: bool,
    mode: DiagnosticsMode,
) -> Result<()> {
    let session = RawExportSession::open(input)?;
    run_diagnostics_report_with_session(&session, output, report, strict, mode)
}

pub(crate) fn run_diagnostics_report_with_session(
    session: &RawExportSession,
    output: Option<&Path>,
    report: &Path,
    strict: bool,
    mode: DiagnosticsMode,
) -> Result<()> {
    let started = Instant::now();
    let mut warnings = Vec::new();
    let mut blocked = Vec::new();
    let summary = summarize_raw_export(session, &mut warnings, &mut blocked)?;
    let raw_export_abi = validate_raw_export_abi_with_session(session)?;
    let native_ui_export_abi = validate_native_ui_export_abi_with_session(session)?;
    if let Some(output) = output {
        write_raw_export_abi_validation_report_with_session(session, output, false)?;
        write_native_ui_export_abi_validation_report_with_session(session, output, false)?;
    }
    blocked.extend(
        raw_export_abi
            .missing_required_files
            .iter()
            .map(|entry| format!("raw export ABI missing required file: {}", entry)),
    );
    blocked.extend(
        raw_export_abi
            .missing_declared_files
            .iter()
            .map(|entry| format!("raw export ABI missing declared file: {}", entry)),
    );
    blocked.extend(
        raw_export_abi
            .path_violations
            .iter()
            .map(|entry| format!("raw export ABI path violation: {}", entry)),
    );
    if native_ui_export_abi.missing_report {
        blocked.push("native UI export ABI validation report is missing".to_string());
    }
    blocked.extend(
        native_ui_export_abi
            .schema_violations
            .iter()
            .map(|entry| format!("native UI export ABI schema violation: {}", entry)),
    );
    blocked.extend(
        native_ui_export_abi
            .path_violations
            .iter()
            .map(|entry| format!("native UI export ABI path violation: {}", entry)),
    );
    blocked.extend(
        native_ui_export_abi
            .contract_violations
            .iter()
            .map(|entry| format!("native UI export ABI contract violation: {}", entry)),
    );
    let runtime = summarize_runtime_output(output)?;
    session.verify_input_generation()?;

    fs::create_dir_all(report.parent().unwrap_or_else(|| Path::new(".")))?;
    write_report(
        report,
        &CompilerReport {
            schema_version: "neonei/rust-compiler-report/current",
            mode: mode.as_str().to_string(),
            input: normalize_path(session.authority_input()),
            input_authority: session.input_authority().as_str(),
            resolved_input: normalize_path(session.input()),
            output: output.map(normalize_path),
            elapsed_ms: started.elapsed().as_millis(),
            manifest_io: session.manifest_io_metrics(),
            input_generation_id: session.generation_id().to_string(),
            raw_export: summary,
            raw_export_abi,
            native_ui_export_abi,
            runtime,
            warnings,
            blocked: blocked.clone(),
        },
    )?;

    if strict && !blocked.is_empty() {
        return Err(anyhow!(
            "Raw Export {} blocked; see {}",
            mode.as_str(),
            report.display()
        ));
    }

    Ok(())
}
