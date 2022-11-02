use std::io::Write;

use crate::Result;
use anyhow::anyhow;
use async_trait::async_trait;
use clap::Parser;
use github_scbot_core::sentry::send_test_event;

use super::{Command, CommandContext};

/// Send a test message to Sentry
#[derive(Parser)]
pub(crate) struct DebugTestSentryCommand {
    /// Custom message, defaults to "This is a test"
    #[clap(short, long)]
    message: Option<String>,
}

#[async_trait(?Send)]
impl Command for DebugTestSentryCommand {
    async fn execute<W: Write>(self, ctx: CommandContext<W>) -> Result<()> {
        if ctx.config.sentry_url.is_empty() {
            Err(anyhow!("Sentry URL is not configured."))
        } else {
            send_test_event(&ctx.config.sentry_url, self.message).await;
            Ok(())
        }
    }
}
