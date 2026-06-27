use crate::cli::{Cli, Command};
use crate::diagnostics::{run_diagnostics_report, DiagnosticsMode};
use crate::schemas::write_schema_catalog;
use crate::stages::run_compile_kernel;
use anyhow::Result;

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
            run_compile_kernel(&input, &output, scope, strict, debug_json)?;
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
