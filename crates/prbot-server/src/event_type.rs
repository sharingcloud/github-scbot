//! Event types.

use std::convert::TryFrom;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum EventTypeError {
    /// Unsupported event.
    #[error("Unsupported event: {}", event)]
    UnsupportedEvent { event: String },
}

/// Event type.
#[derive(Debug, Clone, Copy)]
pub enum EventType {
    /// Check suite event.
    CheckSuite,
    /// Issue comment event.
    IssueComment,
    /// Ping event.
    Ping,
    /// Pull request event.
    PullRequest,
    /// Pull request review event.
    PullRequestReview,
}

impl EventType {
    /// Convert event type to static str.
    pub fn to_str(self) -> &'static str {
        self.into()
    }
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_str())
    }
}

impl TryFrom<&str> for EventType {
    type Error = EventTypeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "check_suite" => Ok(Self::CheckSuite),
            "issue_comment" => Ok(Self::IssueComment),
            "ping" => Ok(Self::Ping),
            "pull_request" => Ok(Self::PullRequest),
            "pull_request_review" => Ok(Self::PullRequestReview),
            name => Err(EventTypeError::UnsupportedEvent {
                event: name.to_owned(),
            }),
        }
    }
}

impl From<EventType> for &'static str {
    fn from(event_type: EventType) -> Self {
        match event_type {
            EventType::CheckSuite => "check_suite",
            EventType::IssueComment => "issue_comment",
            EventType::Ping => "ping",
            EventType::PullRequest => "pull_request",
            EventType::PullRequestReview => "pull_request_review",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EventType;

    #[test]
    fn test_event_as_str() {
        assert_eq!(EventType::Ping.to_str(), "ping");
        assert_eq!(EventType::PullRequest.to_str(), "pull_request");
    }
}
