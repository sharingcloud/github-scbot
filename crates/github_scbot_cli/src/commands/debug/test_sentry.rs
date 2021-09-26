use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_conf::sentry::with_sentry_configuration;
use sentry_core::{protocol::Event, Hub, Level};
use stable_eyre::eyre::{eyre, Result};

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
            with_sentry_configuration(&ctx.config, || async {
                // Create event
                let event = Event {
                    message: Some(self.message.unwrap_or_else(|| "This is a test".into())),
                    level: Level::Info,
                    ..Default::default()
                };

                Hub::with_active(|hub| hub.capture_event(event));
                Ok(())
            })
            .await
        }
    }
}
