use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "elysium-compiler")]
#[command(about = "Elysium runtime data compiler", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Inspect a Raw Export and emit a deterministic diagnostics report without compiling.
    Inspect {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        report: PathBuf,
    },
    /// Validate Raw Export and optional dist-data contracts, failing on blockers.
    Validate {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        report: PathBuf,
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Emit the stable compiler schema/catalog contract.
    Schemas {
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Compile raw-export data into Elysium runtime packs.
    Compile {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        report: PathBuf,
        #[arg(long, value_enum, default_value_t = CompileScope::All)]
        scope: CompileScope,
        #[arg(long)]
        threads: Option<usize>,
        #[arg(long, default_value_t = false)]
        strict: bool,
        /// Emit large JSON debug packs next to binary runtime packs.
        #[arg(long, default_value_t = false)]
        debug_json: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum CompileScope {
    All,
    NativeUi,
    Search,
    Browser,
    Recipes,
    Ui,
    Textures,
}

impl CompileScope {
    pub fn as_str(self) -> &'static str {
        crate::compiler_scope_catalog::compile_scope_name(self)
    }
}
