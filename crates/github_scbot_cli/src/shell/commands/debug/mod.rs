//! Debug commands.

use argh::FromArgs;
use async_trait::async_trait;
use stable_eyre::eyre::Result;

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
    async fn execute<'a>(self, ctx: CommandContext<'a>) -> Result<()> {
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
    async fn execute<'a>(self, ctx: CommandContext<'a>) -> Result<()> {
        match self {
            Self::TestSentry(sub) => sub.execute(ctx).await,
        }
    }
}
