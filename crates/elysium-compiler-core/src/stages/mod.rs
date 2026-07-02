use crate::cli::CompileScope;
use crate::kernel::{
    trace_report_path, CompileKernelContext, CompileKernelModule, CompileStage,
    CompileStageContract, CompileStageRegistry,
};
use crate::native_ui_export_abi::write_native_ui_export_abi_validation_report;
use crate::native_ui_report::compile_native_ui_layout_report;
use crate::pack_abi::{purge_out_of_scope_runtime_artifacts, write_pack_abi_validation_report};
use crate::packs::browser::compile_browser_pack;
use crate::packs::recipe::compile_recipe_pack;
use crate::packs::search::compile_search_pack;
use crate::packs::texture::compile_texture_pack;
use crate::packs::ui::compile_ui_pack;
use crate::raw_export_abi::write_raw_export_abi_validation_report;
use crate::recipe_domain::captured_ui_family_key;
use crate::runtime::{compile_runtime_reports, purge_debug_json_artifacts};
use crate::ui_pack_abi::write_ui_pack_abi_validation_report;
use crate::validation::compile_semantic_validation_report;
use anyhow::{Context, Result};
use serde_json::json;
use std::fs;
use std::path::Path;

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
        let _module_name = module.name();
        module.register_stages(&mut registry);
    }
    let kernel = registry.into_kernel();
    let mut context = CompileKernelContext::new(input, output, scope, strict, debug_json);
    kernel.run(&mut context)
}

fn compile_kernel_modules() -> Vec<CompileKernelModule> {
    vec![
        CompileKernelModule::new("compiler.lifecycle", register_lifecycle_stages),
        CompileKernelModule::new("compiler.raw_export_abi", register_raw_export_abi_stages),
        CompileKernelModule::new(
            "compiler.native_ui_export_abi",
            register_native_ui_export_abi_stages,
        ),
        CompileKernelModule::new("compiler.runtime_packs", register_runtime_pack_stages),
        CompileKernelModule::new(
            "compiler.semantic_validation",
            register_semantic_validation_stages,
        ),
        CompileKernelModule::new(
            "compiler.native_ui_validation",
            register_native_ui_validation_stages,
        ),
        CompileKernelModule::new(
            "compiler.pack_abi_validation",
            register_pack_abi_validation_stages,
        ),
        CompileKernelModule::new("compiler.runtime_reports", register_runtime_report_stages),
        CompileKernelModule::new("compiler.trace", register_trace_stages),
    ]
}

fn register_lifecycle_stages(registry: &mut CompileStageRegistry) {
    registry.register(CompileStage::module_stage(
        "compiler.lifecycle",
        "prepare-output",
        CompileStageContract::new(
            &["compile-options"],
            &["output-directory", "debug-json-policy"],
            &["compiler.lifecycle.prepare"],
        ),
        prepare_output_stage,
    ));
}

fn register_raw_export_abi_stages(registry: &mut CompileStageRegistry) {
    registry.register(CompileStage::module_stage(
        "compiler.raw_export_abi",
        "validate-raw-export-abi",
        CompileStageContract::new(
            &["raw-export-manifest"],
            &["rust/raw-export-abi-validation-report.json"],
            &["compiler.raw_export_abi_validator"],
        ),
        raw_export_abi_validation_stage,
    ));
}

fn register_native_ui_export_abi_stages(registry: &mut CompileStageRegistry) {
    registry.register(CompileStage::module_stage(
        "compiler.native_ui_export_abi",
        "validate-native-ui-export-abi",
        CompileStageContract::new(
            &["validation/native-ui-abi.json", "raw-export-manifest"],
            &["rust/native-ui-export-abi-validation-report.json"],
            &["compiler.native_ui_export_abi_validator"],
        ),
        native_ui_export_abi_validation_stage,
    ));
}

fn register_runtime_pack_stages(registry: &mut CompileStageRegistry) {
    registry.register(CompileStage::module_stage(
        "compiler.runtime_packs",
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
    ));
}

fn register_semantic_validation_stages(registry: &mut CompileStageRegistry) {
    registry.register(CompileStage::module_stage(
        "compiler.semantic_validation",
        "validate-semantic-contract",
        CompileStageContract::new(
            &["raw-export-reports", "browser-contract", "semantic-reports"],
            &["rust/semantic-validation-report.json"],
            &["compiler.semantic_validator"],
        ),
        semantic_validation_stage,
    ));
}

fn register_native_ui_validation_stages(registry: &mut CompileStageRegistry) {
    registry.register(CompileStage::module_stage(
        "compiler.native_ui_validation",
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
    ));
    registry.register(CompileStage::module_stage(
        "compiler.native_ui_validation",
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
    ));
}

fn register_pack_abi_validation_stages(registry: &mut CompileStageRegistry) {
    registry.register(CompileStage::module_stage(
        "compiler.pack_abi_validation",
        "validate-pack-abi",
        CompileStageContract::new(
            &["binary-runtime-packs", "runtime-report-inputs"],
            &["rust/pack-validation-report.json"],
            &["compiler.pack_abi_validator"],
        ),
        pack_abi_validation_stage,
    ));
}

fn register_runtime_report_stages(registry: &mut CompileStageRegistry) {
    registry.register(CompileStage::module_stage(
        "compiler.runtime_reports",
        "emit-runtime-reports",
        CompileStageContract::new(
            &["pack-abi-validation-report", "runtime-artifact-registry"],
            &["runtime-manifest", "integrity-report", "dist-manifest"],
            &["compiler.runtime_manifest"],
        ),
        runtime_reports_stage,
    ));
}

fn register_trace_stages(registry: &mut CompileStageRegistry) {
    registry.register(CompileStage::module_stage(
        "compiler.trace",
        "emit-kernel-trace",
        CompileStageContract::new(
            &["compile-stage-events"],
            &["rust/compile-kernel-trace.json"],
            &["compiler.tracepoints"],
        ),
        kernel_trace_stage,
    ));
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
    match context.scope {
        CompileScope::All => {
            compile_browser_pack(
                context.input,
                context.output,
                context.strict,
                context.debug_json,
            )?;
            compile_recipe_pack(
                context.input,
                context.output,
                context.strict,
                context.debug_json,
            )?;
            compile_ui_pack(
                context.input,
                context.output,
                context.strict,
                context.debug_json,
            )?;
            compile_texture_pack(
                context.input,
                context.output,
                context.strict,
                context.debug_json,
            )?;
        }
        CompileScope::NativeUi => {
            compile_browser_pack(
                context.input,
                context.output,
                context.strict,
                context.debug_json,
            )?;
            compile_recipe_pack(
                context.input,
                context.output,
                context.strict,
                context.debug_json,
            )?;
            compile_ui_pack(
                context.input,
                context.output,
                context.strict,
                context.debug_json,
            )?;
        }
        CompileScope::Search => compile_search_pack(
            context.input,
            context.output,
            context.strict,
            context.debug_json,
        )?,
        CompileScope::Browser => compile_browser_pack(
            context.input,
            context.output,
            context.strict,
            context.debug_json,
        )?,
        CompileScope::Recipes => compile_recipe_pack(
            context.input,
            context.output,
            context.strict,
            context.debug_json,
        )?,
        CompileScope::Ui => compile_ui_pack(
            context.input,
            context.output,
            context.strict,
            context.debug_json,
        )?,
        CompileScope::Textures => compile_texture_pack(
            context.input,
            context.output,
            context.strict,
            context.debug_json,
        )?,
    }
    Ok(())
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
            "schemaVersion": "elysium-compiler/compile-kernel-trace/v1",
            "scope": context.scope.as_str(),
            "strict": context.strict,
            "debugJson": context.debug_json,
            "stages": context.events(),
        }))?,
    )?;
    Ok(())
}
