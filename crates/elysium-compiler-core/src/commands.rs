use crate::cli::{Cli, Command, CompileScope};
use crate::diagnostics::{run_diagnostics_report, DiagnosticsMode};
use crate::packs::browser::compile_browser_pack;
use crate::packs::recipe::compile_recipe_pack;
use crate::packs::search::compile_search_pack;
use crate::packs::texture::compile_texture_pack;
use crate::packs::ui::compile_ui_pack;
use crate::recipe_domain::captured_ui_family_key;
use crate::runtime::{compile_runtime_reports, purge_debug_json_artifacts};
use crate::schemas::write_schema_catalog;
use crate::validation::compile_semantic_validation_report;
use anyhow::{Context, Result};
use std::fs;

pub fn run_command(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Inspect {
            input,
            report,
            threads,
        } => {
            configure_threads(threads);
            run_diagnostics_report(&input, None, &report, false, DiagnosticsMode::Inspect)
        }
        Command::Validate {
            input,
            report,
            output,
            threads,
        } => {
            configure_threads(threads);
            run_diagnostics_report(
                &input,
                output.as_deref(),
                &report,
                true,
                DiagnosticsMode::Validate,
            )
        }
        Command::Schemas { output } => write_schema_catalog(output.as_deref()),
        Command::Compile {
            input,
            output,
            report,
            scope,
            threads,
            strict,
            debug_json,
        } => {
            configure_threads(threads);
            fs::create_dir_all(&output)
                .with_context(|| format!("create output directory {}", output.display()))?;
            if !debug_json {
                purge_debug_json_artifacts(&output)?;
            }
            match scope {
                CompileScope::All => {
                    compile_browser_pack(&input, &output, strict, debug_json)?;
                    compile_recipe_pack(&input, &output, strict, debug_json)?;
                    compile_ui_pack(&input, &output, strict, debug_json)?;
                    compile_texture_pack(&input, &output, strict, debug_json)?;
                }
                CompileScope::NativeUi => {
                    compile_browser_pack(&input, &output, strict, debug_json)?;
                    compile_recipe_pack(&input, &output, strict, debug_json)?;
                    compile_ui_pack(&input, &output, strict, debug_json)?;
                }
                CompileScope::Search => compile_search_pack(&input, &output, strict, debug_json)?,
                CompileScope::Browser => compile_browser_pack(&input, &output, strict, debug_json)?,
                CompileScope::Recipes => compile_recipe_pack(&input, &output, strict, debug_json)?,
                CompileScope::Ui => compile_ui_pack(&input, &output, strict, debug_json)?,
                CompileScope::Textures => {
                    compile_texture_pack(&input, &output, strict, debug_json)?
                }
            }
            compile_semantic_validation_report(&input, &output)?;
            compile_runtime_reports(&output, scope, strict, debug_json, captured_ui_family_key)?;
            run_diagnostics_report(
                &input,
                Some(&output),
                &report,
                strict,
                DiagnosticsMode::Compile,
            )
        }
    }
}

fn configure_threads(threads: Option<usize>) {
    if let Some(threads) = threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .ok();
    }
}
