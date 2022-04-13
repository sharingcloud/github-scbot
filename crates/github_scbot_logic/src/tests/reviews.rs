//! Review tests

use github_scbot_conf::Config;
use github_scbot_database2::Result;
use github_scbot_ghapi::adapter::{DummyAPIAdapter, IAPIAdapter};
use github_scbot_redis::{DummyRedisAdapter, IRedisAdapter, LockInstance, LockStatus};
use github_scbot_types::{
    common::{GhUser, GhUserPermission},
    reviews::{GhReview, GhReviewState},
};

use crate::{
    commands::{CommandExecutor, CommandParser},
    status::PullRequestStatus,
    summary::SummaryTextGenerator,
    LogicError, Result as LogicResult,
};

// async fn arrange(
//     conf: &Config,
//     db_adapter: &dyn IDatabaseAdapter,
// ) -> (RepositoryModel, PullRequestModel) {
//     // Create a repository and a pull request
//     let repo = RepositoryModel::builder(conf, "me", "TestRepo")
//         .create_or_update(db_adapter.repository())
//         .await
//         .unwrap();

//     let pr = PullRequestModel::builder(&repo, 1, "me")
//         .name("PR 1")
//         .create_or_update(db_adapter.pull_request())
//         .await
//         .unwrap();

//     (repo, pr)
// }

#[actix_rt::test]
async fn test_review_creation() -> Result<()> {
    // async fn parse_and_execute_command(
    //     config: &Config,
    //     api_adapter: &dyn IAPIAdapter,
    //     db_adapter: &dyn IDatabaseAdapter,
    //     redis_adapter: &dyn IRedisAdapter,
    //     repo: &mut RepositoryModel,
    //     pr: &mut PullRequestModel,
    //     command_str: &str,
    // ) -> LogicResult<()> {
    //     // Parse comment
    //     let commands = CommandParser::parse_commands(config, command_str);
    //     CommandExecutor::execute_commands(
    //         config,
    //         api_adapter,
    //         db_adapter,
    //         redis_adapter,
    //         repo,
    //         pr,
    //         0,
    //         "me",
    //         commands,
    //     )
    //     .await?;

    //     Ok(())
    // }

    // using_test_db("test_logic_reviews", |config, pool| async move {
    //     let db_adapter = DatabaseAdapter::new(pool);
    //     let mut api_adapter = DummyAPIAdapter::new();
    //     let mut redis_adapter = DummyRedisAdapter::new();
    //     api_adapter
    //         .user_permissions_get_response
    //         .set_callback(Box::new(|_| Ok(GhUserPermission::Write)));

    //     let (mut repo, mut pr) = arrange(&config, &db_adapter).await;

    //     // Simulate review
    //     let review = GhReview {
    //         state: GhReviewState::Pending,
    //         submitted_at: Some(chrono::Utc::now()),
    //         user: GhUser {
    //             login: "me".to_string(),
    //         },
    //     };

    //     let instance = LockInstance::new_dummy("pouet");
    //     redis_adapter
    //         .try_lock_resource_response
    //         .set_callback(Box::new(move |_| {
    //             Ok(LockStatus::SuccessfullyLocked(instance.clone()))
    //         }));

    //     // Simulate another review
    //     let review2 = GhReview {
    //         state: GhReviewState::ChangesRequested,
    //         submitted_at: Some(chrono::Utc::now()),
    //         user: GhUser {
    //             login: "him".to_string(),
    //         },
    //     };

    //     // List reviews
    //     let reviews = pr.reviews(db_adapter.review()).await.unwrap();
    //     assert_eq!(reviews[0].username(), "me");
    //     assert_eq!(reviews[1].username(), "him");
    //     assert!(!reviews[1].required());

    //     // Parse comment
    //     parse_and_execute_command(
    //         &config,
    //         &api_adapter,
    //         &db_adapter,
    //         &redis_adapter,
    //         &mut repo,
    //         &mut pr,
    //         "test-bot req+ @him",
    //     )
    //     .await?;

    //     // Retrieve "him" review
    //     let review = db_adapter
    //         .review()
    //         .get_from_pull_request_and_username(&repo, &pr, "him")
    //         .await
    //         .unwrap();
    //     assert!(review.required());

    //     // Parse comment
    //     parse_and_execute_command(
    //         &config,
    //         &api_adapter,
    //         &db_adapter,
    //         &redis_adapter,
    //         &mut repo,
    //         &mut pr,
    //         "test-bot req- @him",
    //     )
    //     .await?;

    //     // Lock PR
    //     parse_and_execute_command(
    //         &config,
    //         &api_adapter,
    //         &db_adapter,
    //         &redis_adapter,
    //         &mut repo,
    //         &mut pr,
    //         "test-bot lock+",
    //     )
    //     .await?;

    //     // Retrieve "him" review
    //     let review = db_adapter
    //         .review()
    //         .get_from_pull_request_and_username(&repo, &pr, "him")
    //         .await
    //         .unwrap();
    //     assert!(!review.required());

    //     // Generate status
    //     let status = PullRequestStatus::from_database(&api_adapter, &db_adapter, &repo, &pr)
    //         .await
    //         .unwrap();
    //     assert!(status.approved_reviewers.is_empty());
    //     assert!(!status.automerge);
    //     assert_eq!(
    //         status.needed_reviewers_count,
    //         repo.default_needed_reviewers_count() as usize
    //     );
    //     assert!(status.missing_required_reviewers.is_empty());
    //     assert!(status.locked);

    //     // Generate status comment
    //     let comment = SummaryTextGenerator::generate(&status).unwrap();
    //     assert!(!comment.is_empty());

    //     Ok::<_, LogicError>(())
    // })
    // .await
    todo!()
}
