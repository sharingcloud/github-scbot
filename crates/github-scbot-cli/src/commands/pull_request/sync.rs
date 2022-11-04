use std::io::Write;

use crate::Result;
use async_trait::async_trait;
use clap::Parser;
use github_scbot_core::types::repository::RepositoryPath;
use github_scbot_domain::{pulls::PullRequestLogic, status::StatusLogic};

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
        .await?;

        let upstream_pr = ctx
            .api_adapter
            .pulls_get(repo_owner, repo_name, pr_number)
            .await?;

        StatusLogic::update_pull_request_status(
            ctx.api_adapter.as_ref(),
            ctx.db_adapter.as_ref(),
            ctx.redis_adapter.as_ref(),
            repo_owner,
            repo_name,
            pr_number,
            &upstream_pr,
        )
        .await?;

        writeln!(
            ctx.writer,
            "Pull request #{} from '{}' updated from GitHub.",
            self.number, self.repository_path
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use futures_util::FutureExt;
    use github_scbot_core::types::pulls::GhPullRequest;
    use github_scbot_core::types::rule_branch::RuleBranch;
    use github_scbot_database::{
        MockMergeRuleDB, MockPullRequestDB, MockRepositoryDB, MockRequiredReviewerDB, PullRequest,
        Repository,
    };
    use github_scbot_redis::{LockInstance, LockStatus};
    use mockall::predicate;

    use crate::testutils::{test_command, CommandContextTest};
    use crate::Result;

    #[actix_rt::test]
    async fn test() -> Result<()> {
        let mut ctx = CommandContextTest::new();
        ctx.api_adapter
            .expect_pulls_get()
            .times(1)
            .return_once(|_, _, _| {
                Ok(GhPullRequest {
                    number: 1,
                    ..Default::default()
                })
            });
        ctx.api_adapter
            .expect_pull_reviews_list()
            .times(1)
            .return_once(|_, _, _| Ok(vec![]));
        ctx.api_adapter
            .expect_issue_labels_list()
            .times(1)
            .return_once(|_, _, _| Ok(vec![]));
        ctx.api_adapter
            .expect_issue_labels_replace_all()
            .times(1)
            .return_once(|_, _, _, _| Ok(()));
        ctx.api_adapter
            .expect_comments_post()
            .times(1)
            .return_once(|_, _, _, _| Ok(1));
        ctx.api_adapter
            .expect_commit_statuses_update()
            .times(1)
            .return_once(|_, _, _, _, _, _| Ok(()));

        ctx.redis_adapter
            .expect_wait_lock_resource()
            .times(1)
            .returning(|_, _| {
                Ok(LockStatus::SuccessfullyLocked(LockInstance::new_dummy(
                    "test",
                )))
            });

        ctx.db_adapter.expect_repositories().returning(|| {
            let mut mock = MockRepositoryDB::new();
            mock.expect_get()
                .with(predicate::eq("owner"), predicate::eq("name"))
                .returning(|_, _| {
                    async { Ok(Some(Repository::builder().build().unwrap())) }.boxed()
                });

            Box::new(mock)
        });
        ctx.db_adapter.expect_merge_rules().returning(|| {
            let mut mock = MockMergeRuleDB::new();
            mock.expect_get()
                .with(
                    predicate::eq("owner"),
                    predicate::eq("name"),
                    predicate::eq(RuleBranch::Named("".into())),
                    predicate::eq(RuleBranch::Named("".into())),
                )
                .returning(|_, _, _, _| async { Ok(None) }.boxed());

            Box::new(mock)
        });
        ctx.db_adapter.expect_required_reviewers().returning(|| {
            let mut mock = MockRequiredReviewerDB::new();
            mock.expect_list()
                .with(
                    predicate::eq("owner"),
                    predicate::eq("name"),
                    predicate::eq(1),
                )
                .returning(|_, _, _| async { Ok(vec![]) }.boxed());

            Box::new(mock)
        });
        ctx.db_adapter.expect_pull_requests().returning(|| {
            let mut mock = MockPullRequestDB::new();
            mock.expect_get()
                .with(
                    predicate::eq("owner"),
                    predicate::eq("name"),
                    predicate::eq(1),
                )
                .returning(|_, _, _| {
                    async { Ok(Some(PullRequest::builder().build().unwrap())) }.boxed()
                });

            mock.expect_set_status_comment_id()
                .with(
                    predicate::eq("owner"),
                    predicate::eq("name"),
                    predicate::eq(1),
                    predicate::eq(1),
                )
                .returning(|_, _, _, _| {
                    async { Ok(PullRequest::builder().build().unwrap()) }.boxed()
                });

            Box::new(mock)
        });

        let output = test_command(ctx, &["pull-requests", "sync", "owner/name", "1"]).await?;
        assert_eq!(
            output,
            "Pull request #1 from 'owner/name' updated from GitHub.\n"
        );

        Ok(())
    }
}
