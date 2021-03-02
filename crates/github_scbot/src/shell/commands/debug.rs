//! Debug commands.

use github_scbot_conf::{Config, sentry::with_sentry_configuration};
use super::errors::{CommandError, Result};
use sentry_core::{Hub, Level, protocol::Event};

pub(crate) fn send_test_event_to_sentry(config: &Config, message: Option<String>) -> Result<()> {
    if config.sentry_url.is_empty() {
        Err(CommandError::SentryNotConfigured)
    } else {
        with_sentry_configuration(&config, || {
            // Create event
            let event = Event {
                message: Some(message.unwrap_or_else(|| "This is a test".into())),
                level: Level::Info,
                ..Default::default()
            };

            Hub::with_active(|hub| hub.capture_event(event));
            Ok(())
        })
    }
}
