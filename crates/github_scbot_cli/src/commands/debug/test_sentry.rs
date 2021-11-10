use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_sentry::{
    eyre::{eyre::eyre, Result},
    send_test_event,
};

use super::{Command, CommandContext};

/// send a test message to Sentry.
#[derive(FromArgs)]
#[argh(subcommand, name = "test-sentry")]
pub(crate) struct DebugTestSentryCommand {
    /// custom message, defaults to "This is a test".
    #[argh(option, short = 'm')]
    message: Option<String>,
}

#[async_trait(?Send)]
impl Command for DebugTestSentryCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        if ctx.config.sentry_url.is_empty() {
            Err(eyre!("Sentry URL is not configured."))
        } else {
            send_test_event(&ctx.config.sentry_url, self.message).await;
            Ok(())
        }
    }
}
