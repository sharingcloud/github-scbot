//! Debug commands.

use std::io::Write;

use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_sentry::eyre::Result;

use super::{Command, CommandContext};

mod test_sentry;

use test_sentry::DebugTestSentryCommand;

/// debug related commands.
#[derive(FromArgs)]
#[argh(subcommand, name = "debug")]
pub(crate) struct DebugCommand {
    #[argh(subcommand)]
    inner: DebugSubCommand,
}

#[async_trait(?Send)]
impl Command for DebugCommand {
    async fn execute<W: Write>(self, ctx: CommandContext<W>) -> Result<()> {
        self.inner.execute(ctx).await
    }
}

#[derive(FromArgs)]
#[argh(subcommand)]
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
