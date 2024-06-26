use anyhow::anyhow;
use async_trait::async_trait;
use clap::Parser;
use prbot_sentry::send_test_event;

use super::{Command, CommandContext};
use crate::Result;

/// Send a test message to Sentry
#[derive(Parser)]
pub(crate) struct DebugTestSentryCommand {
    /// Custom message, defaults to "This is a test"
    #[clap(short, long)]
    message: Option<String>,
}

#[async_trait]
impl Command for DebugTestSentryCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        if ctx.config.sentry.url.is_empty() {
            Err(anyhow!("Sentry URL is not configured."))
        } else {
            send_test_event(self.message).await;
            Ok(())
        }
    }
}
