//! Descriptor-owned compiler command catalog and ops table.
//!
//! The public command surface is a stable ABI exposed to NeoNEI and release
//! tooling. Keep command names, required-command policy, schema projection, and
//! execution dispatch in one table so the CLI cannot drift from the published
//! capability contract.

use crate::cli::Command;
use crate::diagnostics::{run_diagnostics_report, DiagnosticsMode};
use crate::schemas::write_schema_catalog;
use crate::stages::run_compile_kernel;
use anyhow::{bail, Result};
use serde_json::{json, Value};
use std::collections::BTreeSet;

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
    configure_threads(*threads);
    run_compile_kernel(input, output, *scope, *strict, *debug_json)?;
    run_diagnostics_report(
        input,
        Some(output),
        report,
        *strict,
        DiagnosticsMode::Compile,
    )
}

fn run_inspect_command(command: &Command) -> Result<()> {
    let Command::Inspect {
        input,
        report,
        threads,
    } = command
    else {
        bail!(
            "inspect command descriptor received {}",
            command_name(command)
        );
    };
    configure_threads(*threads);
    run_diagnostics_report(input, None, report, false, DiagnosticsMode::Inspect)
}

fn run_validate_command(command: &Command) -> Result<()> {
    let Command::Validate {
        input,
        report,
        output,
        threads,
    } = command
    else {
        bail!(
            "validate command descriptor received {}",
            command_name(command)
        );
    };
    configure_threads(*threads);
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

fn configure_threads(threads: Option<usize>) {
    if let Some(threads) = threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .ok();
    }
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
