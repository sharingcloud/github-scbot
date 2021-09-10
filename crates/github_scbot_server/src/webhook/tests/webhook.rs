//! Webhook handler tests

use github_scbot_types::{
    checks::{
        GhCheckConclusion, GhCheckStatus, GhCheckSuite, GhCheckSuiteAction, GhCheckSuiteEvent,
    },
    common::{GhApplication, GhBranch, GhBranchShort, GhLabel, GhRepository, GhUser},
    issues::{GhIssue, GhIssueComment, GhIssueCommentAction, GhIssueCommentEvent, GhIssueState},
    ping::GhPingEvent,
    pulls::{
        GhPullRequest, GhPullRequestAction, GhPullRequestEvent, GhPullRequestShort,
        GhPullRequestState,
    },
    reviews::{GhReview, GhReviewAction, GhReviewEvent, GhReviewState},
};
use pretty_assertions::assert_eq;

use super::fixtures;
use crate::{
    webhook::{
        checks::parse_check_suite_event, issues::parse_issue_comment_event, ping::parse_ping_event,
        pulls::parse_pull_request_event, reviews::parse_review_event,
    },
    Result as ServerResult,
};

#[test]
fn test_ping_event_parsing() -> ServerResult<()> {
    assert_eq!(
        parse_ping_event(fixtures::PING_EVENT_DATA)?,
        GhPingEvent {
            zen: "Favor focus over features.".to_string(),
            hook_id: 12_345_678,
            repository: Some(GhRepository {
                name: "test-repo".to_string(),
                full_name: "Example/test-repo".to_string(),
                owner: GhUser {
                    login: "Example".to_string()
                }
            }),
            sender: Some(GhUser {
                login: "Example".to_string()
            })
        }
    );

    Ok(())
}

#[test]
fn test_check_suite_completed_event_parsing() -> ServerResult<()> {
    assert_eq!(
        parse_check_suite_event(fixtures::CHECK_SUITE_COMPLETED_DATA)?,
        GhCheckSuiteEvent {
            action: GhCheckSuiteAction::Completed,
            check_suite: GhCheckSuite {
                id: 12_345_678,
                head_branch: "head-branch".to_string(),
                head_sha: "12345678123456781234567812345678".to_string(),
                status: GhCheckStatus::Completed,
                conclusion: Some(GhCheckConclusion::Failure),
                pull_requests: vec![GhPullRequestShort {
                    number: 1214,
                    head: GhBranchShort {
                        reference: "head-branch".to_string(),
                        sha: "12345678123456781234567812345678".to_string(),
                    },
                    base: GhBranchShort {
                        reference: "stable".to_string(),
                        sha: "12345678123456781234567812345678".to_string(),
                    }
                }],
                app: GhApplication {
                    slug: "github-actions".to_string(),
                    owner: GhUser {
                        login: "github".to_string()
                    },
                    name: "GitHub Actions".to_string()
                },
                created_at: chrono::DateTime::parse_from_rfc3339("2020-11-13T17:34:29Z")
                    .expect("bad date")
                    .with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339("2020-11-13T17:41:47Z")
                    .expect("bad date")
                    .with_timezone(&chrono::Utc),
            },
            repository: GhRepository {
                name: "test-repo".to_string(),
                full_name: "Example/test-repo".to_string(),
                owner: GhUser {
                    login: "Example".to_string()
                }
            },
            organization: GhUser {
                login: "Example".to_string()
            },
            sender: GhUser {
                login: "me".to_string()
            },
        }
    );

    Ok(())
}

#[test]
fn test_issue_comment_created_event_parsing() -> ServerResult<()> {
    assert_eq!(
        parse_issue_comment_event(fixtures::ISSUE_COMMENT_CREATED_DATA)?,
        GhIssueCommentEvent {
            action: GhIssueCommentAction::Created,
            changes: None,
            issue: GhIssue {
                number: 1,
                title: "Add the webhook module".to_string(),
                user: GhUser {
                    login: "me".to_string()
                },
                labels: vec![],
                state: GhIssueState::Open,
                created_at: chrono::DateTime::parse_from_rfc3339("2020-11-15T15:49:48Z")
                    .expect("bad date")
                    .with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339("2020-11-15T16:13:15Z")
                    .expect("bad date")
                    .with_timezone(&chrono::Utc),
                closed_at: None,
                body: Some("Ajout du module de gestion des webhooks.".to_string())
            },
            comment: GhIssueComment {
                id: 12_345_678,
                user: GhUser {
                    login: "me".to_string()
                },
                created_at: chrono::DateTime::parse_from_rfc3339("2020-11-15T16:13:15Z")
                    .expect("bad date")
                    .with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339("2020-11-15T16:13:15Z")
                    .expect("bad date")
                    .with_timezone(&chrono::Utc),
                body: "Un autre commentaire de test.".to_string()
            },
            repository: GhRepository {
                name: "github-scbot".to_string(),
                full_name: "Example/github-scbot".to_string(),
                owner: GhUser {
                    login: "Example".to_string()
                }
            },
            organization: GhUser {
                login: "Example".to_string()
            },
            sender: GhUser {
                login: "me".to_string()
            }
        }
    );

    Ok(())
}

#[test]
fn test_pull_request_opened_event_parsing() -> ServerResult<()> {
    assert_eq!(
        parse_pull_request_event(fixtures::PULL_REQUEST_OPENED_DATA)?,
        GhPullRequestEvent {
            action: GhPullRequestAction::Opened,
            number: 1214,
            pull_request: GhPullRequest {
                number: 1214,
                state: GhPullRequestState::Open,
                locked: false,
                title: "This is a PR".to_string(),
                user: GhUser {
                    login: "me".to_string()
                },
                body: Some("Ceci est\nle corps de la \nPR".to_string()),
                created_at: chrono::DateTime::parse_from_rfc3339("2020-11-13T17:34:23Z")
                    .expect("bad date")
                    .with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339("2020-11-13T17:34:23Z")
                    .expect("bad date")
                    .with_timezone(&chrono::Utc),
                closed_at: None,
                merged_at: None,
                requested_reviewers: vec![],
                labels: vec![],
                draft: false,
                head: GhBranch {
                    label: Some("this-is-a-label".to_string()),
                    reference: "this-is-a-ref".to_string(),
                    sha: "123456789123456789123456789".to_string(),
                    user: Some(GhUser {
                        login: "Example".to_string()
                    }),
                },
                base: GhBranch {
                    label: Some("Example:stable".to_string()),
                    reference: "stable".to_string(),
                    sha: "123456789123456789123456789".to_string(),
                    user: Some(GhUser {
                        login: "Example".to_string()
                    }),
                },
                merged: Some(false),
                mergeable: None,
                rebaseable: None,
            },
            label: None,
            requested_reviewer: None,
            repository: GhRepository {
                name: "test-repo".to_string(),
                full_name: "Example/test-repo".to_string(),
                owner: GhUser {
                    login: "Example".to_string()
                },
            },
            organization: GhUser {
                login: "Example".to_string()
            },
            sender: GhUser {
                login: "me".to_string()
            }
        }
    );

    Ok(())
}

#[test]
fn test_pull_request_labeled_event_parsing() -> ServerResult<()> {
    assert_eq!(
        parse_pull_request_event(fixtures::PULL_REQUEST_LABELED_DATA)?,
        GhPullRequestEvent {
            action: GhPullRequestAction::Labeled,
            number: 1214,
            pull_request: GhPullRequest {
                number: 1214,
                state: GhPullRequestState::Open,
                locked: false,
                title: "This is a PR".to_string(),
                user: GhUser {
                    login: "me".to_string()
                },
                body: Some("This is a PR body".to_string()),
                created_at: chrono::DateTime::parse_from_rfc3339("2020-11-13T17:34:23Z")
                    .expect("bad date")
                    .with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339("2020-11-13T17:39:42Z")
                    .expect("bad date")
                    .with_timezone(&chrono::Utc),
                closed_at: None,
                merged_at: None,
                requested_reviewers: vec![GhUser {
                    login: "reviewer1".to_string()
                }],
                labels: vec![
                    GhLabel {
                        name: "lang/python".to_string(),
                        color: "beffaf".to_string(),
                        description: Some("Contains Python code".to_string()),
                    },
                    GhLabel {
                        name: "step/awaiting-changes".to_string(),
                        color: "e58082".to_string(),
                        description: Some("Waiting for changes".to_string()),
                    }
                ],
                draft: false,
                head: GhBranch {
                    label: Some("label-test".to_string()),
                    reference: "ref-test".to_string(),
                    sha: "123456789123456789123456789".to_string(),
                    user: Some(GhUser {
                        login: "Example".to_string()
                    }),
                },
                base: GhBranch {
                    label: Some("Example:stable".to_string()),
                    reference: "stable".to_string(),
                    sha: "123456789123456789123456789".to_string(),
                    user: Some(GhUser {
                        login: "Example".to_string()
                    }),
                },
                merged: Some(false),
                mergeable: Some(true),
                rebaseable: Some(true),
            },
            label: Some(GhLabel {
                name: "step/awaiting-changes".to_string(),
                color: "e58082".to_string(),
                description: Some("Waiting for changes".to_string())
            }),
            requested_reviewer: None,
            repository: GhRepository {
                name: "test-repo".to_string(),
                full_name: "Example/test-repo".to_string(),
                owner: GhUser {
                    login: "Example".to_string()
                },
            },
            organization: GhUser {
                login: "Example".to_string()
            },
            sender: GhUser {
                login: "bot".to_string()
            }
        }
    );

    Ok(())
}

#[test]
fn test_review_submitted_event_parsing() -> ServerResult<()> {
    assert_eq!(
        parse_review_event(fixtures::PULL_REQUEST_REVIEW_SUBMITTED_DATA)?,
        GhReviewEvent {
            action: GhReviewAction::Submitted,
            review: GhReview {
                user: GhUser {
                    login: "me".to_string()
                },
                submitted_at: chrono::DateTime::parse_from_rfc3339("2020-11-13T17:25:46Z")
                    .expect("bad date")
                    .with_timezone(&chrono::Utc),
                state: GhReviewState::ChangesRequested
            },
            pull_request: GhPullRequest {
                number: 1206,
                state: GhPullRequestState::Open,
                locked: false,
                title: "This is a PR".to_string(),
                user: GhUser {
                    login: "orig".to_string()
                },
                body: Some("This is a PR body".to_string()),
                created_at: chrono::DateTime::parse_from_rfc3339("2020-11-12T16:09:47Z")
                    .expect("bad date")
                    .with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339("2020-11-13T17:25:46Z")
                    .expect("bad date")
                    .with_timezone(&chrono::Utc),
                closed_at: None,
                merged_at: None,
                requested_reviewers: vec![
                    GhUser {
                        login: "reviewer1".to_string(),
                    },
                    GhUser {
                        login: "reviewer2".to_string(),
                    },
                    GhUser {
                        login: "reviewer3".to_string(),
                    }
                ],
                labels: vec![
                    GhLabel {
                        name: "lang/html".to_string(),
                        color: "beffaf".to_string(),
                        description: Some("Contains HTML code".to_string())
                    },
                    GhLabel {
                        name: "step/awaiting-review".to_string(),
                        color: "6cdd9b".to_string(),
                        description: Some("Waiting for review".to_string())
                    }
                ],
                draft: false,
                head: GhBranch {
                    label: Some("label".to_string()),
                    reference: "ref".to_string(),
                    sha: "123456789123456789123456789".to_string(),
                    user: Some(GhUser {
                        login: "Example".to_string()
                    })
                },
                base: GhBranch {
                    label: Some("Example:stable".to_string()),
                    reference: "stable".to_string(),
                    sha: "123456789123456789123456789".to_string(),
                    user: Some(GhUser {
                        login: "Example".to_string()
                    })
                },
                merged: None,
                mergeable: None,
                rebaseable: None
            },
            repository: GhRepository {
                name: "test-repo".to_string(),
                full_name: "Example/test-repo".to_string(),
                owner: GhUser {
                    login: "Example".to_string()
                }
            },
            organization: GhUser {
                login: "Example".to_string()
            },
            sender: GhUser {
                login: "me".to_string()
            }
        }
    );

    Ok(())
}
