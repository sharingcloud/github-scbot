//! Review tests

use crate::{
    database::{
        establish_single_connection,
        models::{
            PullRequestCreation, PullRequestModel, RepositoryCreation, RepositoryModel, ReviewModel,
        },
        DbConn,
    },
    types::{PullRequestReview, PullRequestReviewState, User},
    utils::test_init,
    webhook::logic::{commands::parse_comment, reviews::handle_review},
};

fn arrange(conn: &DbConn) -> (RepositoryModel, PullRequestModel) {
    // Create a repository and a pull request
    let repo = RepositoryModel::create(
        &conn,
        RepositoryCreation {
            name: "TestRepo",
            owner: "me",
        },
    )
    .unwrap();
    let pr = PullRequestModel::create(
        &conn,
        PullRequestCreation {
            repository_id: repo.id,
            name: "PR 1",
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

    let conn = establish_single_connection().unwrap();
    let (repo, mut pr) = arrange(&conn);

    // Simulate review
    let review = PullRequestReview {
        id: 1,
        body: "OK".to_string(),
        commit_id: "1234".to_string(),
        state: PullRequestReviewState::Pending,
        submitted_at: chrono::Utc::now(),
        user: User {
            id: 1,
            login: "me".to_string(),
        },
    };
    handle_review(&conn, &pr, &review).await.unwrap();

    // Simulate another review
    let review2 = PullRequestReview {
        id: 2,
        body: "OK".to_string(),
        commit_id: "1234".to_string(),
        state: PullRequestReviewState::ChangesRequested,
        submitted_at: chrono::Utc::now(),
        user: User {
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

    // Retrieve "him" review
    let review = ReviewModel::get_from_pull_request_and_username(&conn, pr.id, "him").unwrap();
    assert_eq!(review.required, false);
}
