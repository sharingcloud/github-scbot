//! Debug commands.

use std::io::Write;

use async_trait::async_trait;
use clap::{Parser, Subcommand};

use super::{Command, CommandContext};

use crate::Result;
mod test_sentry;
use test_sentry::DebugTestSentryCommand;

/// debug related commands.
#[derive(Parser)]
pub(crate) struct DebugCommand {
    #[clap(subcommand)]
    inner: DebugSubCommand,
}

#[async_trait(?Send)]
impl Command for DebugCommand {
    async fn execute<W: Write>(self, ctx: CommandContext<W>) -> Result<()> {
        self.inner.execute(ctx).await
    }
}

#[derive(Subcommand)]
enum DebugSubCommand {
    TestSentry(DebugTestSentryCommand),
}

#[async_trait(?Send)]
impl Command for DebugSubCommand {
    async fn execute<W: Write>(self, ctx: CommandContext<W>) -> Result<()> {
        match self {
            Self::TestSentry(sub) => sub.execute(ctx).await,
        }
    }
}
