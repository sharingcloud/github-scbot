use std::path::PathBuf;

use async_trait::async_trait;
use clap::Parser;

use super::{Command, CommandContext};
use crate::Result;

/// Convert a PEM file to a one-line string
#[derive(Parser)]
pub(crate) struct PemToStringCommand {
    /// PEM file
    pem_file: PathBuf,
}

#[async_trait]
impl Command for PemToStringCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let content = tokio::fs::read_to_string(self.pem_file).await?;
        writeln!(
            ctx.writer.write().await,
            "{}",
            content.lines().collect::<Vec<_>>().join("\\n")
        )?;

        Ok(())
    }
}
