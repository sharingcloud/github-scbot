//! Utils commands.

mod pem_to_string;

use async_trait::async_trait;
use clap::{Parser, Subcommand};

use self::pem_to_string::PemToStringCommand;
use super::{Command, CommandContext};
use crate::Result;

/// Utils related commands
#[derive(Parser)]
pub(crate) struct UtilsCommand {
    #[clap(subcommand)]
    inner: UtilsSubCommand,
}

#[async_trait]
impl Command for UtilsCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        self.inner.execute(ctx).await
    }
}

#[derive(Subcommand)]
enum UtilsSubCommand {
    PemToString(PemToStringCommand),
}

#[async_trait]
impl Command for UtilsSubCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        match self {
            Self::PemToString(sub) => sub.execute(ctx).await,
        }
    }
}
