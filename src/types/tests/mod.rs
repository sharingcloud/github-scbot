use super::events::EventType;

#[test]
fn test_event_as_str() {
    assert_eq!(EventType::Ping.to_str(), "ping");
    assert_eq!(EventType::PullRequest.to_str(), "pull_request");
    assert_eq!(
        EventType::PullRequestReviewComment.to_str(),
        "pull_request_review_comment"
    );
}
