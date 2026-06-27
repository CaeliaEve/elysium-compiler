use crate::cli::CompileScope;
use crate::kernel::{trace_report_path, CompileKernel, CompileKernelContext, CompileStage};
use crate::packs::browser::compile_browser_pack;
use crate::packs::recipe::compile_recipe_pack;
use crate::packs::search::compile_search_pack;
use crate::packs::texture::compile_texture_pack;
use crate::packs::ui::compile_ui_pack;
use crate::recipe_domain::captured_ui_family_key;
use crate::runtime::{compile_runtime_reports, purge_debug_json_artifacts};
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
    let kernel = CompileKernel::new(vec![
        CompileStage::new("prepare-output", prepare_output_stage),
        CompileStage::new("emit-runtime-packs", emit_runtime_packs_stage),
        CompileStage::new("validate-semantic-contract", semantic_validation_stage),
        CompileStage::new("emit-runtime-reports", runtime_reports_stage),
        CompileStage::new("emit-kernel-trace", kernel_trace_stage),
    ]);
    let mut context = CompileKernelContext::new(input, output, scope, strict, debug_json);
    kernel.run(&mut context)
}

fn prepare_output_stage(context: &mut CompileKernelContext<'_>) -> Result<()> {
    fs::create_dir_all(context.output)
        .with_context(|| format!("create output directory {}", context.output.display()))?;
    if !context.debug_json {
        purge_debug_json_artifacts(context.output)?;
    }
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

fn semantic_validation_stage(context: &mut CompileKernelContext<'_>) -> Result<()> {
    compile_semantic_validation_report(context.input, context.output)
}

fn runtime_reports_stage(context: &mut CompileKernelContext<'_>) -> Result<()> {
    compile_runtime_reports(
        context.output,
        context.scope,
        context.strict,
        context.debug_json,
        captured_ui_family_key,
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
