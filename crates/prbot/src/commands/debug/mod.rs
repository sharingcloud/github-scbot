//! Debug commands.

use async_trait::async_trait;
use clap::{Parser, Subcommand};

use super::{Command, CommandContext};
use crate::Result;
mod test_sentry;
use test_sentry::DebugTestSentryCommand;

/// Debug related commands
#[derive(Parser)]
pub(crate) struct DebugCommand {
    #[clap(subcommand)]
    inner: DebugSubCommand,
}

#[async_trait]
impl Command for DebugCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        self.inner.execute(ctx).await
    }
}

#[derive(Subcommand)]
enum DebugSubCommand {
    TestSentry(DebugTestSentryCommand),
}

#[async_trait]
impl Command for DebugSubCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        match self {
            Self::TestSentry(sub) => sub.execute(ctx).await,
        }
    }
}
