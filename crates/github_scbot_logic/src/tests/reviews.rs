//! Review tests

use github_scbot_conf::Config;
use github_scbot_database::{
    establish_single_test_connection,
    models::{PullRequestModel, RepositoryModel, ReviewModel},
    DbConn,
};
use github_scbot_types::{
    common::GHUser,
    pulls::GHMergeStrategy,
    reviews::{GHReview, GHReviewState},
};

use super::test_config;
use crate::{
    commands::parse_commands,
    reviews::handle_review,
    status::{generate_pr_status_comment, PullRequestStatus},
};

fn arrange(conf: &Config, conn: &DbConn) -> (RepositoryModel, PullRequestModel) {
    // Create a repository and a pull request
    let repo = RepositoryModel::builder(&conf, "me", "TestRepo")
        .create_or_update(&conn)
        .unwrap();

    let pr = PullRequestModel::builder(&repo, 1, "me")
        .name("PR 1")
        .create_or_update(&conn)
        .unwrap();

    (repo, pr)
}

#[actix_rt::test]
async fn test_review_creation() {
    let config = test_config();

    let conn = establish_single_test_connection(&config).unwrap();
    let (repo, mut pr) = arrange(&config, &conn);

    // Simulate review
    let review = GHReview {
        state: GHReviewState::Pending,
        submitted_at: chrono::Utc::now(),
        user: GHUser {
            login: "me".to_string(),
        },
    };
    handle_review(&config, &conn, &repo, &pr, &review)
        .await
        .unwrap();

    // Simulate another review
    let review2 = GHReview {
        state: GHReviewState::ChangesRequested,
        submitted_at: chrono::Utc::now(),
        user: GHUser {
            login: "him".to_string(),
        },
    };
    handle_review(&config, &conn, &repo, &pr, &review2)
        .await
        .unwrap();

    // List reviews
    let reviews = pr.get_reviews(&conn).unwrap();
    assert_eq!(reviews[0].username, "me");
    assert_eq!(reviews[1].username, "him");
    assert_eq!(reviews[1].required, false);

    // Parse comment
    parse_commands(
        &config,
        &conn,
        &repo,
        &mut pr,
        0,
        "me",
        "test-bot req+ @him",
    )
    .await
    .unwrap();

    // Retrieve "him" review
    let review = ReviewModel::get_from_pull_request_and_username(&conn, &repo, &pr, "him").unwrap();
    assert_eq!(review.required, true);

    // Parse comment
    parse_commands(
        &config,
        &conn,
        &repo,
        &mut pr,
        0,
        "me",
        "test-bot req- @him",
    )
    .await
    .unwrap();

    // Lock PR
    parse_commands(&config, &conn, &repo, &mut pr, 0, "me", "test-bot lock+")
        .await
        .unwrap();

    // Retrieve "him" review
    let review = ReviewModel::get_from_pull_request_and_username(&conn, &repo, &pr, "him").unwrap();
    assert_eq!(review.required, false);

    // Generate status
    let reviews = pr.get_reviews(&conn).unwrap();
    let status = PullRequestStatus::from_pull_request(&repo, &pr, &reviews).unwrap();
    assert!(status.approved_reviewers.is_empty());
    assert!(!status.automerge);
    assert_eq!(status.needed_reviewers_count, 2);
    assert!(status.missing_required_reviewers.is_empty());
    assert_eq!(status.locked, true);

    // Generate status comment
    let comment = generate_pr_status_comment(&repo, &pr, &reviews, GHMergeStrategy::Merge).unwrap();
    assert!(!comment.is_empty());
}
