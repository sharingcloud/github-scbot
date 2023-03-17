//! CLI module.

use anyhow::Result;
use args::{Args, CommandExecutor};
use clap::Parser;
use github_scbot_core::config::configure_startup;

pub(crate) mod args;
mod commands;
#[cfg(test)]
mod testutils;
pub(crate) mod utils;

/// Initialize command line.
pub fn initialize_command_line() -> Result<()> {
    let config = configure_startup()?;
    let args = Args::parse();

    CommandExecutor::parse_args(config, args)
}
