use crate::cli::Cli;
use crate::compiler_command_catalog::run_compiler_command;
use anyhow::Result;

pub fn run_command(cli: Cli) -> Result<()> {
    run_compiler_command(&cli.command)
}
