//! Webhook event types

use std::error::Error;

#[derive(Debug)]
pub enum EventType {
    CheckRun,
    CheckSuite,
    IssueComment,
    Ping,
    PullRequest,
    PullRequestReview,
    PullRequestReviewComment,
    Push,
    Status,
}

impl EventType {
    pub fn try_from_str(name: &str) -> Result<Self, Box<dyn Error>> {
        match name {
            "check_run" => Ok(EventType::CheckRun),
            "check_suite" => Ok(EventType::CheckSuite),
            "issue_comment" => Ok(EventType::IssueComment),
            "ping" => Ok(EventType::Ping),
            "pull_request" => Ok(EventType::PullRequest),
            "pull_request_review" => Ok(EventType::PullRequestReview),
            "pull_request_review_comment" => Ok(EventType::PullRequestReviewComment),
            "push" => Ok(EventType::Push),
            "status" => Ok(EventType::Status),
            name => Err(format!("Unsupported event name {}", name).into()),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
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
    use super::*;

    #[test]
    fn test_event_as_str() {
        assert_eq!(EventType::Ping.as_str(), "ping");
        assert_eq!(EventType::PullRequest.as_str(), "pull_request");
        assert_eq!(
            EventType::PullRequestReviewComment.as_str(),
            "pull_request_review_comment"
        );
    }
}
