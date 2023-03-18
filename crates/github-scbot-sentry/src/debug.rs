use github_scbot_config::Config;
use sentry::{protocol::Event, Hub, Level};

use super::with_sentry_configuration;

pub async fn send_test_event(config: &Config, message: Option<String>) {
    with_sentry_configuration(config, || async {
        // Create event
        let event = Event {
            message: Some(message.unwrap_or_else(|| "This is a test".into())),
            level: Level::Info,
            ..Default::default()
        };

        Hub::with_active(|hub| {
            let uuid = hub.capture_event(event);
            println!("Event UUID: {}", uuid);
            uuid
        });

        Ok::<(), ()>(())
    })
    .await
    .unwrap()
}
