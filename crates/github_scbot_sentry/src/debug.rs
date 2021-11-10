use sentry::{protocol::Event, Hub, Level};

use crate::with_sentry_configuration;

pub async fn send_test_event(sentry_url: &str, message: Option<String>) {
    with_sentry_configuration(sentry_url, || async {
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
