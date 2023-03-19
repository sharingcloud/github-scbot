use sentry::{protocol::Event, Hub, Level};

pub async fn send_test_event(message: Option<String>) {
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
}
