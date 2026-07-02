use crate::io::normalize_path;
use crate::manifest::read_manifest;
use crate::native_ui_export_abi::{
    validate_native_ui_export_abi, write_native_ui_export_abi_validation_report,
};
use crate::raw_export::summarize_raw_export;
use crate::raw_export_abi::{validate_raw_export_abi, write_raw_export_abi_validation_report};
use crate::reports::{summarize_runtime_output, write_report, CompilerReport};
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
    let started = Instant::now();
    let manifest = read_manifest(input)?;
    let mut warnings = Vec::new();
    let mut blocked = Vec::new();
    let summary = summarize_raw_export(input, &manifest, &mut warnings, &mut blocked)?;
    let raw_export_abi = validate_raw_export_abi(input)?;
    let native_ui_export_abi = validate_native_ui_export_abi(input)?;
    if let Some(output) = output {
        write_raw_export_abi_validation_report(input, output, false)?;
        write_native_ui_export_abi_validation_report(input, output, false)?;
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

    fs::create_dir_all(report.parent().unwrap_or_else(|| Path::new(".")))?;
    write_report(
        report,
        &CompilerReport {
            schema_version: "neonei/rust-compiler-report/current",
            mode: mode.as_str().to_string(),
            input: normalize_path(input),
            output: output.map(normalize_path),
            elapsed_ms: started.elapsed().as_millis(),
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
