//! Review tests

use chrono::Utc;
use github_scbot_conf::Config;
use github_scbot_database2::{
    use_temporary_db, DbService, DbServiceImplPool, PullRequest, Repository, Result,
};
use github_scbot_ghapi::adapter::{ApiService, GhReviewApi, GhReviewStateApi, MockApiService};
use github_scbot_redis::{MockRedisService, RedisService};
use github_scbot_types::{
    common::{GhUser, GhUserPermission},
    pulls::GhPullRequest,
};
use mockall::predicate;

use crate::{
    commands::{CommandExecutor, CommandParser},
    Result as LogicResult,
};

async fn arrange(conf: &Config, db_adapter: &dyn DbService) -> (Repository, PullRequest) {
    // Create a repository and a pull request
    let repo = Repository::builder()
        .with_config(conf)
        .name("name")
        .owner("owner")
        .default_enable_checks(false)
        .build()
        .unwrap();
    let repo = db_adapter
        .repositories()
        .create(repo.clone())
        .await
        .unwrap();

    let pr = PullRequest::builder()
        .with_repository(&repo)
        .status_comment_id(1u64)
        .number(1u64)
        .build()
        .unwrap();
    let pr = db_adapter.pull_requests().create(pr.clone()).await.unwrap();

    (repo, pr)
}

#[actix_rt::test]
async fn test_review_creation() -> Result<()> {
    #[allow(clippy::too_many_arguments)]
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

    use_temporary_db(config, "test_reviews_creation", |config, pool| async move {
        let db_adapter = DbServiceImplPool::new(pool);
        let mut api_adapter = MockApiService::new();
        let redis_adapter = MockRedisService::new();

        // One call to get PR
        api_adapter
            .expect_pulls_get()
            .times(1)
            .return_const(Ok(GhPullRequest {
                ..Default::default()
            }));

        // One call for command sender, one for target reviewer
        api_adapter
            .expect_user_permissions_get()
            .times(2)
            .return_const(Ok(GhUserPermission::Write));

        // One call to assign reviewer
        api_adapter
            .expect_pull_reviewer_requests_add()
            .times(1)
            .return_const(Ok(()));

        // One call to list upstream reviews
        api_adapter
            .expect_pull_reviews_list()
            .times(1)
            .return_const(Ok(vec![GhReviewApi {
                user: GhUser {
                    login: "him".into(),
                },
                state: GhReviewStateApi::Pending,
                submitted_at: Utc::now(),
            }]));

        // One call to get check suites
        api_adapter
            .expect_check_suites_list()
            .times(1)
            .return_const(Ok(vec![]));

        // One call to list labels
        api_adapter
            .expect_issue_labels_list()
            .times(1)
            .return_const(Ok(vec![]));

        // One call to replace all labels
        api_adapter
            .expect_issue_labels_replace_all()
            .times(1)
            .with(
                predicate::eq("owner"),
                predicate::eq("name"),
                predicate::eq(1),
                predicate::function(|v| v == ["step/awaiting-required-review"]),
            )
            .return_const(Ok(()));

        // One call to update target comment
        api_adapter
            .expect_comments_update()
            .times(1)
            .return_const(Ok(0));

        // One call to update status
        api_adapter
            .expect_commit_statuses_update()
            .times(1)
            .return_const(Ok(()));

        // One call to post comment
        api_adapter
            .expect_comments_post()
            .times(1)
            .return_const(Ok(1));

        let (repo, pr) = arrange(&config, &db_adapter).await;
        let upstream_pr = GhPullRequest {
            mergeable: Some(true),
            ..Default::default()
        };

        parse_and_execute_command(
            &config,
            &api_adapter,
            &db_adapter,
            &redis_adapter,
            repo.owner(),
            repo.name(),
            pr.number(),
            &upstream_pr,
            "bot req+ @him",
        )
        .await?;

        assert!(
            db_adapter
                .required_reviewers()
                .get("owner", "name", 1, "him")
                .await?
                .is_some(),
            "him should be a required reviewer"
        );

        let mut api_adapter = MockApiService::new();

        // One call to get PR
        api_adapter
            .expect_pulls_get()
            .times(1)
            .return_const(Ok(GhPullRequest {
                ..Default::default()
            }));

        // One call for command sender
        api_adapter
            .expect_user_permissions_get()
            .times(1)
            .return_const(Ok(GhUserPermission::Write));

        // One call to unassign reviewer
        api_adapter
            .expect_pull_reviewer_requests_remove()
            .withf(|owner, name, issue_number, reviewers| {
                owner == "owner" && name == "name" && *issue_number == 1u64 && reviewers == ["him"]
            })
            .times(1)
            .return_const(Ok(()));

        // One call to list upstream reviews
        api_adapter
            .expect_pull_reviews_list()
            .times(1)
            .return_const(Ok(vec![GhReviewApi {
                user: GhUser {
                    login: "him".into(),
                },
                state: GhReviewStateApi::Pending,
                submitted_at: Utc::now(),
            }]));

        // One call to get check suites
        api_adapter
            .expect_check_suites_list()
            .times(1)
            .return_const(Ok(vec![]));

        // One call to list labels
        api_adapter
            .expect_issue_labels_list()
            .times(1)
            .return_const(Ok(vec![]));

        // One call to replace all labels
        api_adapter
            .expect_issue_labels_replace_all()
            .times(1)
            .with(
                predicate::eq("owner"),
                predicate::eq("name"),
                predicate::eq(1),
                predicate::function(|v| v == ["step/awaiting-review"]),
            )
            .return_const(Ok(()));

        // One call to update target comment
        api_adapter
            .expect_comments_update()
            .times(1)
            .return_const(Ok(0));

        // One call to update status
        api_adapter
            .expect_commit_statuses_update()
            .times(1)
            .return_const(Ok(()));

        // One call to post comment
        api_adapter
            .expect_comments_post()
            .times(1)
            .return_const(Ok(1));

        parse_and_execute_command(
            &config,
            &api_adapter,
            &db_adapter,
            &redis_adapter,
            repo.owner(),
            repo.name(),
            pr.number(),
            &upstream_pr,
            "bot req- @him",
        )
        .await?;

        assert!(
            db_adapter
                .required_reviewers()
                .get("owner", "name", 1, "him")
                .await?
                .is_none(),
            "him should NOT be a required reviewer"
        );

        Ok(())
    })
    .await;

    Ok(())
}
