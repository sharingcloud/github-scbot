//! Review tests

use github_scbot_conf::Config;
use github_scbot_database2::{Result, DbService, Repository, PullRequest, use_temporary_db, DbServiceImplPool, };
use github_scbot_ghapi::adapter::{ApiService, GithubApiService};
use github_scbot_redis::{RedisService, RedisServiceImpl, MockRedisService};
use github_scbot_types::{
    common::{GhUser, GhUserPermission},
    reviews::{GhReview, GhReviewState}, pulls::GhPullRequest,
};

use crate::{
    commands::{CommandExecutor, CommandParser},
    status::PullRequestStatus,
    summary::SummaryTextGenerator,
    LogicError, Result as LogicResult,
};

async fn arrange(
    conf: &Config,
    db_adapter: &dyn DbService,
) -> (Repository, PullRequest) {
    // Create a repository and a pull request
    let repo = Repository::builder()
        .with_config(conf)
        .build()
        .unwrap();

    let pr = PullRequest::builder()
        .with_repository(&repo)
        .build()
        .unwrap();

    db_adapter.repositories().create(repo.clone()).await.unwrap();
    db_adapter.pull_requests().create(pr.clone()).await.unwrap();

    (repo, pr)
}

#[actix_rt::test]
async fn test_review_creation() -> Result<()> {
    async fn parse_and_execute_command(
        config: &Config,
        api_adapter: &dyn ApiService,
        db_adapter: &dyn DbService,
        redis_adapter: &dyn RedisService,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        upstream_pr: &GhPullRequest,
        command_str: &str,
    ) -> LogicResult<()> {
        // Parse comment
        let commands = CommandParser::parse_commands(config, command_str);
        CommandExecutor::execute_commands(
            config,
            api_adapter,
            db_adapter,
            redis_adapter,
            repo_owner,
            repo_name,
            pr_number,
            upstream_pr,
            0,
            "me",
            commands,
        )
        .await?;

        Ok(())
    }

    let config = Config::from_env();

    use_temporary_db(config, "test_reviews_creation", |config, pool| async {
        let db_adapter = DbServiceImplPool::new(pool);
        let api_adapter = GithubApiService::new(config);
        let redis_adapter = MockRedisService::new();

        let (repo, pr) = arrange(&config, &db_adapter).await;

        parse_and_execute_command(
            &config,
            &api_adapter,
            &db_adapter,
            &redis_adapter,
            repo.owner(),
            &mut pr,
            "test-bot req+ @him",
        )
        .await?;

        Ok(())
    })
    .await;

    // using_test_db("test_logic_reviews", |config, pool| async move {
    //     let db_adapter = DatabaseAdapter::new(pool);
    //     let mut api_adapter = DummyAPIAdapter::new();
    //     let mut redis_adapter = DummyRedisServiceImpl::new();
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
    Ok(())
}
