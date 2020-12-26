//! Webhook event types

use std::error::Error;

#[derive(Debug, Clone, Copy)]
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
            "check_run" => Ok(Self::CheckRun),
            "check_suite" => Ok(Self::CheckSuite),
            "issue_comment" => Ok(Self::IssueComment),
            "ping" => Ok(Self::Ping),
            "pull_request" => Ok(Self::PullRequest),
            "pull_request_review" => Ok(Self::PullRequestReview),
            "pull_request_review_comment" => Ok(Self::PullRequestReviewComment),
            "push" => Ok(Self::Push),
            "status" => Ok(Self::Status),
            name => Err(format!("Unsupported event name {}", name).into()),
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CheckRun => "check_run",
            Self::CheckSuite => "check_suite",
            Self::IssueComment => "issue_comment",
            Self::Ping => "ping",
            Self::PullRequest => "pull_request",
            Self::PullRequestReview => "pull_request_review",
            Self::PullRequestReviewComment => "pull_request_review_comment",
            Self::Push => "push",
            Self::Status => "status",
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
