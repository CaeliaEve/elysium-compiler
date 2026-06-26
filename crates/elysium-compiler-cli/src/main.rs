use anyhow::Result;
use clap::Parser;
use elysium_compiler_core::cli::Cli;
use elysium_compiler_core::commands::run_command;

fn main() -> Result<()> {
    run_command(Cli::parse())
}
