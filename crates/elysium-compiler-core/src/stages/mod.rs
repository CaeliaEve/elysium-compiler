use crate::cli::CompileScope;
use crate::kernel::{
    trace_report_path, CompileKernelContext, CompileKernelModule, CompileStageContract,
    CompileStageDescriptor, CompileStageRegistry, COMPILE_KERNEL_TRACE_REPORT_PATH,
    COMPILE_KERNEL_TRACE_SCHEMA_VERSION,
};
use crate::native_ui_export_abi::write_native_ui_export_abi_validation_report;
use crate::native_ui_export_abi_catalog::{
    NATIVE_UI_EXPORT_ABI_VALIDATION_REPORT_PATH, NATIVE_UI_VALIDATION_DEFAULT_PATH,
};
use crate::native_ui_report::compile_native_ui_layout_report;
use crate::pack_abi::{purge_out_of_scope_runtime_artifacts, write_pack_abi_validation_report};
use crate::raw_export_abi::write_raw_export_abi_validation_report;
use crate::recipe_domain::captured_ui_family_key;
use crate::runtime::{compile_runtime_reports, purge_debug_json_artifacts};
use crate::runtime_pack_plan::{compile_runtime_packs, runtime_pack_compiler_catalog};
use crate::ui_pack_abi::write_ui_pack_abi_validation_report;
use crate::validation::compile_semantic_validation_report;
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

pub const COMPILE_KERNEL_CATALOG_SCHEMA_VERSION: &str =
    "elysium-compiler/compile-kernel-catalog/v1";

const MODULE_LIFECYCLE: &str = "compiler.lifecycle";
const MODULE_RAW_EXPORT_ABI: &str = "compiler.raw_export_abi";
const MODULE_NATIVE_UI_EXPORT_ABI: &str = "compiler.native_ui_export_abi";
const MODULE_RUNTIME_PACKS: &str = "compiler.runtime_packs";
const MODULE_SEMANTIC_VALIDATION: &str = "compiler.semantic_validation";
const MODULE_NATIVE_UI_VALIDATION: &str = "compiler.native_ui_validation";
const MODULE_PACK_ABI_VALIDATION: &str = "compiler.pack_abi_validation";
const MODULE_RUNTIME_REPORTS: &str = "compiler.runtime_reports";
const MODULE_TRACE: &str = "compiler.trace";

const LIFECYCLE_STAGES: &[CompileStageDescriptor] = &[CompileStageDescriptor::new(
    "prepare-output",
    CompileStageContract::new(
        &["compile-options"],
        &["output-directory", "debug-json-policy"],
        &["compiler.lifecycle.prepare"],
    ),
    prepare_output_stage,
)];

const RAW_EXPORT_ABI_STAGES: &[CompileStageDescriptor] = &[CompileStageDescriptor::new(
    "validate-raw-export-abi",
    CompileStageContract::new(
        &["raw-export-manifest"],
        &["rust/raw-export-abi-validation-report.json"],
        &["compiler.raw_export_abi_validator"],
    ),
    raw_export_abi_validation_stage,
)];

const NATIVE_UI_EXPORT_ABI_STAGES: &[CompileStageDescriptor] = &[CompileStageDescriptor::new(
    "validate-native-ui-export-abi",
    CompileStageContract::new(
        &[NATIVE_UI_VALIDATION_DEFAULT_PATH, "raw-export-manifest"],
        &[NATIVE_UI_EXPORT_ABI_VALIDATION_REPORT_PATH],
        &["compiler.native_ui_export_abi_validator"],
    ),
    native_ui_export_abi_validation_stage,
)];

const RUNTIME_PACK_STAGES: &[CompileStageDescriptor] = &[CompileStageDescriptor::new(
    "emit-runtime-packs",
    CompileStageContract::new(
        &[
            "raw-export-manifest",
            "raw-fact-streams",
            "native-ui-capture-facts",
        ],
        &[
            "binary-runtime-packs",
            "ui-pack-assets",
            "optional-debug-json",
        ],
        &["compiler.binary_pack", "compiler.native_ui_pack"],
    ),
    emit_runtime_packs_stage,
)];

const SEMANTIC_VALIDATION_STAGES: &[CompileStageDescriptor] = &[CompileStageDescriptor::new(
    "validate-semantic-contract",
    CompileStageContract::new(
        &["raw-export-reports", "browser-contract", "semantic-reports"],
        &["rust/semantic-validation-report.json"],
        &["compiler.semantic_validator"],
    ),
    semantic_validation_stage,
)];

const NATIVE_UI_VALIDATION_STAGES: &[CompileStageDescriptor] = &[
    CompileStageDescriptor::new(
        "validate-native-ui-layout",
        CompileStageContract::new(
            &[
                "recipes/handler-layout-index.json",
                "recipes/ui-payload-index.json",
            ],
            &["rust/native-ui-layout-report.json"],
            &["compiler.native_ui_validator"],
        ),
        native_ui_layout_validation_stage,
    ),
    CompileStageDescriptor::new(
        "validate-native-ui-pack-abi",
        CompileStageContract::new(
            &[
                "rust/ui-pack/ui_templates.bin",
                "rust/ui-pack/ui_bindings.bin",
                "rust/ui-pack/ui_strings.bin",
            ],
            &["rust/ui-pack-abi-validation-report.json"],
            &["compiler.native_ui_pack_abi_validator"],
        ),
        native_ui_pack_abi_validation_stage,
    ),
];

const PACK_ABI_VALIDATION_STAGES: &[CompileStageDescriptor] = &[CompileStageDescriptor::new(
    "validate-pack-abi",
    CompileStageContract::new(
        &["binary-runtime-packs", "runtime-report-inputs"],
        &["rust/pack-validation-report.json"],
        &["compiler.pack_abi_validator"],
    ),
    pack_abi_validation_stage,
)];

const RUNTIME_REPORT_STAGES: &[CompileStageDescriptor] = &[CompileStageDescriptor::new(
    "emit-runtime-reports",
    CompileStageContract::new(
        &["pack-abi-validation-report", "runtime-artifact-registry"],
        &["runtime-manifest", "integrity-report", "dist-manifest"],
        &["compiler.runtime_manifest"],
    ),
    runtime_reports_stage,
)];

const TRACE_STAGES: &[CompileStageDescriptor] = &[CompileStageDescriptor::new(
    "emit-kernel-trace",
    CompileStageContract::new(
        &["compile-stage-events"],
        &[COMPILE_KERNEL_TRACE_REPORT_PATH],
        &["compiler.tracepoints"],
    ),
    kernel_trace_stage,
)];

const COMPILE_KERNEL_MODULES: &[CompileKernelModule] = &[
    CompileKernelModule::new(MODULE_LIFECYCLE, LIFECYCLE_STAGES),
    CompileKernelModule::new(MODULE_RAW_EXPORT_ABI, RAW_EXPORT_ABI_STAGES),
    CompileKernelModule::new(MODULE_NATIVE_UI_EXPORT_ABI, NATIVE_UI_EXPORT_ABI_STAGES),
    CompileKernelModule::new(MODULE_RUNTIME_PACKS, RUNTIME_PACK_STAGES),
    CompileKernelModule::new(MODULE_SEMANTIC_VALIDATION, SEMANTIC_VALIDATION_STAGES),
    CompileKernelModule::new(MODULE_NATIVE_UI_VALIDATION, NATIVE_UI_VALIDATION_STAGES),
    CompileKernelModule::new(MODULE_PACK_ABI_VALIDATION, PACK_ABI_VALIDATION_STAGES),
    CompileKernelModule::new(MODULE_RUNTIME_REPORTS, RUNTIME_REPORT_STAGES),
    CompileKernelModule::new(MODULE_TRACE, TRACE_STAGES),
];

pub fn run_compile_kernel(
    input: &Path,
    output: &Path,
    scope: CompileScope,
    strict: bool,
    debug_json: bool,
) -> Result<()> {
    fs::create_dir_all(output)
        .with_context(|| format!("create output directory {}", output.display()))?;
    let mut registry = CompileStageRegistry::new();
    for module in compile_kernel_modules() {
        registry.register_module(*module);
    }
    let kernel = registry.into_kernel();
    let mut context = CompileKernelContext::new(input, output, scope, strict, debug_json);
    kernel.run(&mut context)
}

pub(crate) fn compile_kernel_modules() -> &'static [CompileKernelModule] {
    COMPILE_KERNEL_MODULES
}

pub fn compile_kernel_catalog() -> Value {
    let modules = compile_kernel_modules()
        .iter()
        .map(|module| {
            json!({
                "name": module.name(),
                "stageCount": module.stages().len(),
                "stages": module.stages().iter().map(stage_descriptor_catalog).collect::<Vec<_>>(),
            })
        })
        .collect::<Vec<_>>();
    let stage_count: usize = compile_kernel_modules()
        .iter()
        .map(|module| module.stages().len())
        .sum();

    json!({
        "schemaVersion": COMPILE_KERNEL_CATALOG_SCHEMA_VERSION,
        "moduleCount": compile_kernel_modules().len(),
        "stageCount": stage_count,
        "trace": {
            "schemaVersion": COMPILE_KERNEL_TRACE_SCHEMA_VERSION,
            "path": COMPILE_KERNEL_TRACE_REPORT_PATH
        },
        "policy": {
            "registration": "static-module-descriptor-table",
            "execution": "ordered-fail-closed-stage-pipeline",
            "scope": "stage catalog is stable; individual stage emitters gate artifacts by CompileScope"
        },
        "runtimePackCompilerPolicy": {
            "selection": "descriptor-scope-table",
            "compilers": runtime_pack_compiler_catalog()
        },
        "modules": modules
    })
}

fn stage_descriptor_catalog(stage: &CompileStageDescriptor) -> Value {
    let contract = stage.contract();
    json!({
        "name": stage.name(),
        "contract": {
            "inputs": contract.inputs,
            "outputs": contract.outputs,
            "capabilities": contract.capabilities,
        }
    })
}

fn prepare_output_stage(context: &mut CompileKernelContext<'_>) -> Result<()> {
    fs::create_dir_all(context.output)
        .with_context(|| format!("create output directory {}", context.output.display()))?;
    if !context.debug_json {
        purge_debug_json_artifacts(context.output)?;
    }
    purge_out_of_scope_runtime_artifacts(context.output, context.scope, context.debug_json)?;
    Ok(())
}

fn emit_runtime_packs_stage(context: &mut CompileKernelContext<'_>) -> Result<()> {
    compile_runtime_packs(
        context.input,
        context.output,
        context.scope,
        context.strict,
        context.debug_json,
    )
}

fn raw_export_abi_validation_stage(context: &mut CompileKernelContext<'_>) -> Result<()> {
    write_raw_export_abi_validation_report(context.input, context.output, context.strict)?;
    Ok(())
}

fn native_ui_export_abi_validation_stage(context: &mut CompileKernelContext<'_>) -> Result<()> {
    write_native_ui_export_abi_validation_report(context.input, context.output, context.strict)?;
    Ok(())
}

fn semantic_validation_stage(context: &mut CompileKernelContext<'_>) -> Result<()> {
    compile_semantic_validation_report(context.input, context.output)
}

fn native_ui_layout_validation_stage(context: &mut CompileKernelContext<'_>) -> Result<()> {
    compile_native_ui_layout_report(context.output, captured_ui_family_key)?;
    Ok(())
}

fn native_ui_pack_abi_validation_stage(context: &mut CompileKernelContext<'_>) -> Result<()> {
    write_ui_pack_abi_validation_report(context.output, context.scope, context.strict)?;
    Ok(())
}

fn pack_abi_validation_stage(context: &mut CompileKernelContext<'_>) -> Result<()> {
    write_pack_abi_validation_report(
        context.output,
        context.scope,
        context.debug_json,
        context.strict,
    )?;
    Ok(())
}

fn runtime_reports_stage(context: &mut CompileKernelContext<'_>) -> Result<()> {
    compile_runtime_reports(
        context.output,
        context.scope,
        context.strict,
        context.debug_json,
    )
}

fn kernel_trace_stage(context: &mut CompileKernelContext<'_>) -> Result<()> {
    let path = trace_report_path(context.output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        &path,
        serde_json::to_vec_pretty(&json!({
            "schemaVersion": COMPILE_KERNEL_TRACE_SCHEMA_VERSION,
            "scope": context.scope.as_str(),
            "strict": context.strict,
            "debugJson": context.debug_json,
            "stages": context.events(),
        }))?,
    )?;
    Ok(())
}
