//! Review tests

use github_scbot_conf::Config;
use github_scbot_database::{
    models::{PullRequestModel, RepositoryModel, ReviewModel},
    tests::using_test_db,
    DbConn, DbPool, Result,
};
use github_scbot_types::{
    common::GhUser,
    pulls::GhMergeStrategy,
    reviews::{GhReview, GhReviewState},
};

use super::test_config;
use crate::{
    commands::{execute_commands, parse_commands},
    reviews::handle_review,
    status::{generate_pr_status_comment, PullRequestStatus},
    LogicError, Result as LogicResult,
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
async fn test_review_creation() -> Result<()> {
    let config = test_config();

    async fn parse_and_execute_command(
        config: &Config,
        pool: DbPool,
        repo: &RepositoryModel,
        pr: &mut PullRequestModel,
        command_str: &str,
    ) -> LogicResult<()> {
        // Parse comment
        let commands = parse_commands(&config, command_str)?;
        execute_commands(&config, pool.clone(), repo, pr, 0, "me", commands).await?;

        Ok(())
    }

    using_test_db(&config.clone(), "test_logic_reviews", |pool| async move {
        let conn = pool.get().unwrap();
        let (repo, mut pr) = arrange(&config, &conn);

        // Simulate review
        let review = GhReview {
            state: GhReviewState::Pending,
            submitted_at: chrono::Utc::now(),
            user: GhUser {
                login: "me".to_string(),
            },
        };

        handle_review(&config, &conn, &repo, &pr, &review).await?;

        // Simulate another review
        let review2 = GhReview {
            state: GhReviewState::ChangesRequested,
            submitted_at: chrono::Utc::now(),
            user: GhUser {
                login: "him".to_string(),
            },
        };

        handle_review(&config, &conn, &repo, &pr, &review2).await?;

        // List reviews
        let reviews = pr.get_reviews(&conn).unwrap();
        assert_eq!(reviews[0].username, "me");
        assert_eq!(reviews[1].username, "him");
        assert_eq!(reviews[1].required, false);

        // Parse comment
        parse_and_execute_command(&config, pool.clone(), &repo, &mut pr, "test-bot req+ @him")
            .await?;

        // Retrieve "him" review
        let review =
            ReviewModel::get_from_pull_request_and_username(&conn, &repo, &pr, "him").unwrap();
        assert_eq!(review.required, true);

        // Parse comment
        parse_and_execute_command(&config, pool.clone(), &repo, &mut pr, "test-bot req- @him")
            .await?;

        // Lock PR
        parse_and_execute_command(&config, pool.clone(), &repo, &mut pr, "test-bot lock+").await?;

        // Retrieve "him" review
        let review =
            ReviewModel::get_from_pull_request_and_username(&conn, &repo, &pr, "him").unwrap();
        assert_eq!(review.required, false);

        // Generate status
        let reviews = pr.get_reviews(&conn).unwrap();
        let status = PullRequestStatus::from_pull_request(&repo, &pr, &reviews).unwrap();
        assert!(status.approved_reviewers.is_empty());
        assert!(!status.automerge);
        assert_eq!(
            status.needed_reviewers_count,
            repo.default_needed_reviewers_count as usize
        );
        assert!(status.missing_required_reviewers.is_empty());
        assert_eq!(status.locked, true);

        // Generate status comment
        let comment =
            generate_pr_status_comment(&repo, &pr, &reviews, GhMergeStrategy::Merge).unwrap();
        assert!(!comment.is_empty());

        Ok::<_, LogicError>(())
    })
    .await
}
