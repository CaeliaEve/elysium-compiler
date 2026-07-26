//! Descriptor-owned compiler command catalog and ops table.
//!
//! The public command surface is a stable ABI exposed to NeoNEI and release
//! tooling. Keep command names, required-command policy, schema projection, and
//! execution dispatch in one table so the CLI cannot drift from the published
//! capability contract.

use crate::cli::Command;
use crate::diagnostics::{
    run_diagnostics_report, run_diagnostics_report_with_session, DiagnosticsMode,
};
use crate::io::write_json_value;
use crate::output_generation::OutputGeneration;
use crate::schemas::write_schema_catalog;
use crate::stages::run_compile_kernel;
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

pub const COMPILER_COMMAND_CATALOG_SCHEMA_VERSION: &str = "elysium-compiler/command-catalog/v1";
pub const COMPILER_COMMANDS: &[&str] = &["compile", "inspect", "validate", "schemas"];
pub const REQUIRED_COMPILER_COMMANDS: &[&str] = &["schemas", "validate", "compile"];
pub const SCHEMA_HASH_COMMANDS_INPUT: &str = "commands=compile,inspect,validate,schemas";

const COMMAND_DESCRIPTORS: &[CompilerCommandDescriptor] = &[
    CompilerCommandDescriptor::new(
        "compile",
        true,
        &["raw-export-tree", "compile-options"],
        &["runtime-packs", "compiler-report"],
        &["compiler.compile_kernel", "compiler.diagnostics.compile"],
        run_compile_command,
    ),
    CompilerCommandDescriptor::new(
        "inspect",
        false,
        &["raw-export-tree"],
        &["diagnostics-report"],
        &["compiler.diagnostics.inspect"],
        run_inspect_command,
    ),
    CompilerCommandDescriptor::new(
        "validate",
        true,
        &["raw-export-tree", "optional-runtime-output"],
        &["diagnostics-report"],
        &["compiler.diagnostics.validate"],
        run_validate_command,
    ),
    CompilerCommandDescriptor::new(
        "schemas",
        true,
        &["compiler-binary"],
        &["schema-catalog"],
        &["compiler.schema_catalog"],
        run_schemas_command,
    ),
];

#[derive(Clone, Copy)]
pub struct CompilerCommandDescriptor {
    name: &'static str,
    required: bool,
    inputs: &'static [&'static str],
    outputs: &'static [&'static str],
    capabilities: &'static [&'static str],
    runner: fn(&Command) -> Result<()>,
}

impl CompilerCommandDescriptor {
    const fn new(
        name: &'static str,
        required: bool,
        inputs: &'static [&'static str],
        outputs: &'static [&'static str],
        capabilities: &'static [&'static str],
        runner: fn(&Command) -> Result<()>,
    ) -> Self {
        Self {
            name,
            required,
            inputs,
            outputs,
            capabilities,
            runner,
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn required(&self) -> bool {
        self.required
    }

    pub fn inputs(&self) -> &'static [&'static str] {
        self.inputs
    }

    pub fn outputs(&self) -> &'static [&'static str] {
        self.outputs
    }

    pub fn capabilities(&self) -> &'static [&'static str] {
        self.capabilities
    }

    fn run(&self, command: &Command) -> Result<()> {
        (self.runner)(command)
    }
}

pub fn run_compiler_command(command: &Command) -> Result<()> {
    descriptor_for_command(command)?.run(command)
}

pub fn compiler_command_descriptors() -> &'static [CompilerCommandDescriptor] {
    validate_command_descriptors();
    COMMAND_DESCRIPTORS
}

pub fn compiler_command_catalog() -> Value {
    json!({
        "schemaVersion": COMPILER_COMMAND_CATALOG_SCHEMA_VERSION,
        "dispatchPolicy": "static-command-descriptor-ops-table",
        "legacyFallback": "forbidden",
        "commands": compiler_command_descriptors()
            .iter()
            .map(command_descriptor_catalog)
            .collect::<Vec<_>>(),
        "schemaHashInput": SCHEMA_HASH_COMMANDS_INPUT,
    })
}

pub fn descriptor_for_command(command: &Command) -> Result<&'static CompilerCommandDescriptor> {
    let name = command_name(command);
    compiler_command_descriptors()
        .iter()
        .find(|descriptor| descriptor.name == name)
        .ok_or_else(|| anyhow::anyhow!("missing compiler command descriptor: {name}"))
}

pub fn command_name(command: &Command) -> &'static str {
    match command {
        Command::Compile { .. } => "compile",
        Command::Inspect { .. } => "inspect",
        Command::Validate { .. } => "validate",
        Command::Schemas { .. } => "schemas",
    }
}

fn command_descriptor_catalog(descriptor: &CompilerCommandDescriptor) -> Value {
    json!({
        "name": descriptor.name(),
        "required": descriptor.required(),
        "inputs": descriptor.inputs(),
        "outputs": descriptor.outputs(),
        "capabilities": descriptor.capabilities(),
    })
}

fn run_compile_command(command: &Command) -> Result<()> {
    let Command::Compile {
        input,
        output,
        report,
        scope,
        threads,
        strict,
        debug_json,
    } = command
    else {
        bail!(
            "compile command descriptor received {}",
            command_name(command)
        );
    };
    let generation = OutputGeneration::begin(output)?;
    generation.require_external_receipt_path(report)?;
    let staging_output = generation.staging_path().to_path_buf();
    let session = match run_compile_kernel(
        input,
        &staging_output,
        *scope,
        *threads,
        *strict,
        *debug_json,
    ) {
        Ok(session) => session,
        Err(error) => match preserve_texture_failure_receipt(&staging_output, report, &error) {
            Ok(true) => {
                return Err(error.context(format!(
                    "texture diagnostics preserved at {}",
                    report.display()
                )))
            }
            Ok(false) => return Err(error),
            Err(audit_error) => {
                return Err(error.context(format!(
                    "failed to preserve texture diagnostics at {}: {audit_error:#}",
                    report.display()
                )))
            }
        },
    };
    let generation_report = staging_output.join("compiler-report.json");
    run_diagnostics_report_with_session(
        &session,
        Some(&staging_output),
        &generation_report,
        *strict,
        DiagnosticsMode::Compile,
    )?;
    generation.finalize_generation_report(&generation_report)?;
    let sealed = generation.seal(&session, *scope, *strict, *debug_json)?;
    let receipt = sealed.prepare_receipt(report)?;
    let published = sealed.publish()?;
    published.commit_receipt_after_publish(receipt);
    Ok(())
}

fn preserve_texture_failure_receipt(
    staging_output: &Path,
    report: &Path,
    error: &anyhow::Error,
) -> Result<bool> {
    let missing_report_path = staging_output.join("rust/missing-texture-report.json");
    if !missing_report_path.is_file() {
        return Ok(false);
    }
    let missing_texture_report: Value = serde_json::from_slice(
        &fs::read(&missing_report_path)
            .with_context(|| format!("read {}", missing_report_path.display()))?,
    )
    .with_context(|| format!("parse {}", missing_report_path.display()))?;
    let suspicious_report_path = staging_output.join("rust/suspicious-texture-report.json");
    let suspicious_texture_report = if suspicious_report_path.is_file() {
        Some(
            serde_json::from_slice::<Value>(
                &fs::read(&suspicious_report_path)
                    .with_context(|| format!("read {}", suspicious_report_path.display()))?,
            )
            .with_context(|| format!("parse {}", suspicious_report_path.display()))?,
        )
    } else {
        None
    };
    fs::create_dir_all(report.parent().unwrap_or_else(|| Path::new(".")))?;
    write_json_value(
        report,
        &json!({
            "schemaVersion": "elysium-compiler/compile-failure-receipt/v1",
            "status": "blocked",
            "published": false,
            "error": format!("{error:#}"),
            "diagnostics": {
                "missingTextureReport": missing_texture_report,
                "suspiciousTextureReport": suspicious_texture_report,
            }
        }),
    )?;
    Ok(true)
}

fn run_inspect_command(command: &Command) -> Result<()> {
    let Command::Inspect { input, report } = command else {
        bail!(
            "inspect command descriptor received {}",
            command_name(command)
        );
    };
    run_diagnostics_report(input, None, report, false, DiagnosticsMode::Inspect)
}

fn run_validate_command(command: &Command) -> Result<()> {
    let Command::Validate {
        input,
        report,
        output,
    } = command
    else {
        bail!(
            "validate command descriptor received {}",
            command_name(command)
        );
    };
    run_diagnostics_report(
        input,
        output.as_deref(),
        report,
        true,
        DiagnosticsMode::Validate,
    )
}

fn run_schemas_command(command: &Command) -> Result<()> {
    let Command::Schemas { output } = command else {
        bail!(
            "schemas command descriptor received {}",
            command_name(command)
        );
    };
    write_schema_catalog(output.as_deref())
}

fn validate_command_descriptors() {
    if COMMAND_DESCRIPTORS.is_empty() {
        panic!("compiler command descriptor catalog must not be empty");
    }
    let mut names = BTreeSet::new();
    let mut required_names = BTreeSet::new();
    for descriptor in COMMAND_DESCRIPTORS {
        require_non_empty("compiler command name", descriptor.name);
        require_non_empty_slice(
            "compiler command inputs",
            descriptor.name,
            descriptor.inputs,
        );
        require_non_empty_slice(
            "compiler command outputs",
            descriptor.name,
            descriptor.outputs,
        );
        require_non_empty_slice(
            "compiler command capabilities",
            descriptor.name,
            descriptor.capabilities,
        );
        if !names.insert(descriptor.name) {
            panic!("duplicate compiler command descriptor: {}", descriptor.name);
        }
        if descriptor.required {
            required_names.insert(descriptor.name);
        }
    }
    require_exact_projection("compiler command", COMPILER_COMMANDS, &names);
    require_exact_projection(
        "required compiler command",
        REQUIRED_COMPILER_COMMANDS,
        &required_names,
    );
}

fn require_non_empty(label: &str, value: &str) {
    if value.trim().is_empty() {
        panic!("{label} must be non-empty");
    }
}

fn require_non_empty_slice(label: &str, command: &str, values: &[&str]) {
    if values.is_empty() {
        panic!("{label} must not be empty: {command}");
    }
    let mut seen = BTreeSet::new();
    for value in values {
        require_non_empty(label, value);
        if !seen.insert(*value) {
            panic!("duplicate {label}: {command}:{value}");
        }
    }
}

fn require_exact_projection(label: &str, expected: &[&str], seen: &BTreeSet<&'static str>) {
    for name in expected {
        if !seen.contains(name) {
            panic!("missing {label} descriptor: {name}");
        }
    }
    for name in seen {
        if !expected.contains(name) {
            panic!("unknown {label} descriptor: {name}");
        }
    }
}
