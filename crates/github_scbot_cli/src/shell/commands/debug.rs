//! Debug commands.

use github_scbot_conf::{sentry::with_sentry_configuration, Config};
use github_scbot_libs::sentry_core::{protocol::Event, Hub, Level};

use super::errors::{CommandError, Result};

pub(crate) async fn send_test_event_to_sentry(
    config: &Config,
    message: Option<String>,
) -> Result<()> {
    if config.sentry_url.is_empty() {
        Err(CommandError::SentryNotConfigured)
    } else {
        with_sentry_configuration(&config, || async {
            // Create event
            let event = Event {
                message: Some(message.unwrap_or_else(|| "This is a test".into())),
                level: Level::Info,
                ..Default::default()
            };

            Hub::with_active(|hub| hub.capture_event(event));
            Ok(())
        })
        .await
    }
}
