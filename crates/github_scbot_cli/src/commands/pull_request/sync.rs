use std::io::Write;

use crate::errors::{ApiSnafu, IoSnafu, LogicSnafu};
use crate::Result;
use async_trait::async_trait;
use clap::Parser;
use github_scbot_logic::{pulls::PullRequestLogic, status::StatusLogic};
use github_scbot_types::repository::RepositoryPath;
use snafu::ResultExt;

use crate::commands::{Command, CommandContext};

/// Synchronize pull request from upstream
#[derive(Parser)]
pub(crate) struct PullRequestSyncCommand {
    /// Repository path (e.g. 'MyOrganization/my-project')
    repository_path: RepositoryPath,

    /// Pull request number
    number: u64,
}

#[async_trait(?Send)]
impl Command for PullRequestSyncCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let (repo_owner, repo_name) = self.repository_path.components();
        let pr_number = self.number;

        PullRequestLogic::synchronize_pull_request(
            &ctx.config,
            ctx.db_adapter.as_ref(),
            repo_owner,
            repo_name,
            pr_number,
        )
        .await
        .context(LogicSnafu)?;

        let upstream_pr = ctx
            .api_adapter
            .pulls_get(repo_owner, repo_name, pr_number)
            .await
            .context(ApiSnafu)?;

        StatusLogic::update_pull_request_status(
            ctx.api_adapter.as_ref(),
            ctx.db_adapter.as_ref(),
            ctx.redis_adapter.as_ref(),
            repo_owner,
            repo_name,
            pr_number,
            &upstream_pr,
        )
        .await
        .context(LogicSnafu)?;

        writeln!(
            ctx.writer,
            "Pull request #{} from {} updated from GitHub.",
            self.number, self.repository_path
        )
        .context(IoSnafu)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_conf::Config;
    use github_scbot_database2::{use_temporary_db, DbService, DbServiceImplPool};
    use github_scbot_ghapi::adapter::MockApiService;
    use github_scbot_redis::{LockInstance, LockStatus, MockRedisService};
    use github_scbot_types::pulls::GhPullRequest;

    use crate::testutils::test_command;

    #[actix_rt::test]
    async fn test() {
        let config = Config::from_env();
        use_temporary_db(
            config,
            "test_command_pull_request_sync",
            |config, pool| async move {
                let db_adapter = DbServiceImplPool::new(pool.clone());
                let mut api_adapter = MockApiService::new();
                api_adapter
                    .expect_pulls_get()
                    .times(1)
                    .return_once(|_, _, _| {
                        Ok(GhPullRequest {
                            number: 1,
                            ..Default::default()
                        })
                    });

                api_adapter
                    .expect_pull_reviews_list()
                    .times(1)
                    .return_once(|_, _, _| Ok(vec![]));

                api_adapter
                    .expect_check_suites_list()
                    .times(1)
                    .return_once(|_, _, _| Ok(vec![]));

                api_adapter
                    .expect_issue_labels_list()
                    .times(1)
                    .return_once(|_, _, _| Ok(vec![]));

                api_adapter
                    .expect_issue_labels_replace_all()
                    .times(1)
                    .return_once(|_, _, _, _| Ok(()));

                api_adapter
                    .expect_comments_post()
                    .times(1)
                    .return_once(|_, _, _, _| Ok(1));

                api_adapter
                    .expect_commit_statuses_update()
                    .times(1)
                    .return_once(|_, _, _, _, _, _| Ok(()));

                let mut redis_adapter = MockRedisService::new();
                redis_adapter
                    .expect_wait_lock_resource()
                    .times(1)
                    .returning(|_, _| {
                        Ok(LockStatus::SuccessfullyLocked(LockInstance::new_dummy(
                            "test",
                        )))
                    });

                let output = test_command(
                    config.clone(),
                    Box::new(db_adapter),
                    Box::new(api_adapter),
                    Box::new(redis_adapter),
                    &["pull-requests", "sync", "owner/name", "1"],
                )
                .await?;

                assert_eq!(
                    output,
                    "Pull request #1 from owner/name updated from GitHub.\n"
                );

                let db_adapter = DbServiceImplPool::new(pool.clone());
                assert!(
                    db_adapter
                        .repositories()
                        .get("owner", "name")
                        .await?
                        .is_some(),
                    "repository owner/name should exist"
                );
                assert!(
                    db_adapter
                        .pull_requests()
                        .get("owner", "name", 1)
                        .await?
                        .is_some(),
                    "pull request #1 on repository owner/name should exist"
                );

                Ok(())
            },
        )
        .await;
    }
}
