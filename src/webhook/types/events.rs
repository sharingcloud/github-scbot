//! Webhook event types

use std::error::Error;

#[derive(Debug)]
pub enum EventType {
    Ping,
    Push,
    CheckSuite,
    CheckRun,
    Status,
    PullRequest,
    PullRequestReview,
}

impl EventType {
    pub fn try_from_str(name: &str) -> Result<Self, Box<dyn Error>> {
        match name {
            "ping" => Ok(EventType::Ping),
            "push" => Ok(EventType::Push),
            "check_suite" => Ok(EventType::CheckSuite),
            "check_run" => Ok(EventType::CheckRun),
            "status" => Ok(EventType::Status),
            "pull_request" => Ok(EventType::PullRequest),
            "pull_request_review" => Ok(EventType::PullRequestReview),
            name => Err(format!("Unsupported event name {}", name).into()),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            EventType::Ping => "ping",
            EventType::Push => "push",
            EventType::CheckSuite => "check_suite",
            EventType::CheckRun => "check_run",
            EventType::Status => "status",
            EventType::PullRequest => "pull_request",
            EventType::PullRequestReview => "pull_request_review",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_as_str() {
        assert_eq!(EventType::Ping.as_str(), "ping");
        assert_eq!(EventType::PullRequest.as_str(), "pull_request");
    }
}
