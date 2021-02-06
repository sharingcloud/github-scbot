//! Event types.

use std::convert::TryFrom;

use super::errors::TypeError;

/// Event type.
#[derive(Debug, Clone, Copy)]
pub enum EventType {
    /// Check run event.
    CheckRun,
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
    /// Pull request review comment event.
    PullRequestReviewComment,
    /// Push event.
    Push,
    /// Status event.
    Status,
}

impl EventType {
    /// Convert event type to static str.
    pub fn to_str(self) -> &'static str {
        self.into()
    }
}

impl TryFrom<&str> for EventType {
    type Error = TypeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "check_run" => Ok(Self::CheckRun),
            "check_suite" => Ok(Self::CheckSuite),
            "issue_comment" => Ok(Self::IssueComment),
            "ping" => Ok(Self::Ping),
            "pull_request" => Ok(Self::PullRequest),
            "pull_request_review" => Ok(Self::PullRequestReview),
            "pull_request_review_comment" => Ok(Self::PullRequestReviewComment),
            "push" => Ok(Self::Push),
            "status" => Ok(Self::Status),
            name => Err(TypeError::UnsupportedEvent(name.to_owned())),
        }
    }
}

impl From<EventType> for &'static str {
    fn from(event_type: EventType) -> Self {
        match event_type {
            EventType::CheckRun => "check_run",
            EventType::CheckSuite => "check_suite",
            EventType::IssueComment => "issue_comment",
            EventType::Ping => "ping",
            EventType::PullRequest => "pull_request",
            EventType::PullRequestReview => "pull_request_review",
            EventType::PullRequestReviewComment => "pull_request_review_comment",
            EventType::Push => "push",
            EventType::Status => "status",
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
        assert_eq!(
            EventType::PullRequestReviewComment.to_str(),
            "pull_request_review_comment"
        );
    }
}
