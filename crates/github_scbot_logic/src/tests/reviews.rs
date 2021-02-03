//! Review tests

use github_scbot_database::{
    establish_single_test_connection,
    models::{
        PullRequestCreation, PullRequestModel, RepositoryCreation, RepositoryModel, ReviewModel,
    },
    DbConn,
};
use github_scbot_types::{
    common::GHUser,
    pull_requests::{GHPullRequestReview, GHPullRequestReviewState},
};

use super::test_init;
use crate::{
    commands::parse_comment,
    reviews::handle_review,
    status::{generate_pr_status_comment, PullRequestStatus},
};

fn arrange(conn: &DbConn) -> (RepositoryModel, PullRequestModel) {
    // Create a repository and a pull request
    let repo = RepositoryModel::create(
        &conn,
        RepositoryCreation {
            name: "TestRepo".into(),
            owner: "me".into(),
            ..Default::default()
        },
    )
    .unwrap();
    let pr = PullRequestModel::create(
        &conn,
        PullRequestCreation {
            repository_id: repo.id,
            name: "PR 1".into(),
            number: 1,
            ..Default::default()
        },
    )
    .unwrap();

    (repo, pr)
}

#[actix_rt::test]
async fn test_review_creation() {
    test_init();

    let conn = establish_single_test_connection().unwrap();
    let (repo, mut pr) = arrange(&conn);

    // Simulate review
    let review = GHPullRequestReview {
        id: 1,
        body: "OK".to_string(),
        commit_id: "1234".to_string(),
        state: GHPullRequestReviewState::Pending,
        submitted_at: chrono::Utc::now(),
        user: GHUser {
            id: 1,
            login: "me".to_string(),
        },
    };
    handle_review(&conn, &pr, &review).await.unwrap();

    // Simulate another review
    let review2 = GHPullRequestReview {
        id: 2,
        body: "OK".to_string(),
        commit_id: "1234".to_string(),
        state: GHPullRequestReviewState::ChangesRequested,
        submitted_at: chrono::Utc::now(),
        user: GHUser {
            id: 2,
            login: "him".to_string(),
        },
    };
    handle_review(&conn, &pr, &review2).await.unwrap();

    // List reviews
    let reviews = pr.get_reviews(&conn).unwrap();
    assert_eq!(reviews[0].username, "me");
    assert_eq!(reviews[1].username, "him");
    assert_eq!(reviews[1].required, false);

    // Parse comment
    parse_comment(&conn, &repo, &mut pr, "me", "test-bot req+ @him")
        .await
        .unwrap();

    // Retrieve "him" review
    let review = ReviewModel::get_from_pull_request_and_username(&conn, pr.id, "him").unwrap();
    assert_eq!(review.required, true);

    // Parse comment
    parse_comment(&conn, &repo, &mut pr, "me", "test-bot req- @him")
        .await
        .unwrap();

    // Lock PR
    parse_comment(&conn, &repo, &mut pr, "me", "test-bot lock+")
        .await
        .unwrap();

    // Retrieve "him" review
    let review = ReviewModel::get_from_pull_request_and_username(&conn, pr.id, "him").unwrap();
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
    let comment = generate_pr_status_comment(&repo, &pr, &reviews).unwrap();
    assert!(!comment.is_empty());
}
