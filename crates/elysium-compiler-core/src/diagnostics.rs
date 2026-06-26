use crate::io::normalize_path;
use crate::manifest::read_manifest;
use crate::raw_export::summarize_raw_export;
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
