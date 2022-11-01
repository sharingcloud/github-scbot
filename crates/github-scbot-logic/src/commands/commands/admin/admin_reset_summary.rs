use github_scbot_core::types::issues::GhReactionType;

use async_trait::async_trait;

use crate::{
    commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    status::PullRequestStatus,
    summary::SummaryCommentSender,
    Result,
};

pub struct AdminResetSummaryCommand;

impl AdminResetSummaryCommand {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait(?Send)]
impl BotCommand for AdminResetSummaryCommand {
    async fn handle(&self, ctx: &CommandContext) -> Result<CommandExecutionResult> {
        let status = PullRequestStatus::from_database(
            ctx.api_adapter,
            ctx.db_adapter,
            ctx.repo_owner,
            ctx.repo_name,
            ctx.pr_number,
            ctx.upstream_pr,
        )
        .await?;

        // Reset comment ID
        ctx.db_adapter
            .pull_requests()
            .set_status_comment_id(ctx.repo_owner, ctx.repo_name, ctx.pr_number, 0)
            .await?;

        SummaryCommentSender::create_or_update(
            ctx.api_adapter,
            ctx.db_adapter,
            ctx.redis_adapter,
            ctx.repo_owner,
            ctx.repo_name,
            ctx.pr_number,
            &status,
        )
        .await?;

        Ok(CommandExecutionResult::builder()
            .with_status_update(true)
            .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
            .build())
    }
}

#[cfg(test)]
mod tests {
    use futures_util::FutureExt;
    use github_scbot_database::{
        MockMergeRuleDB, MockPullRequestDB, MockRepositoryDB, MockRequiredReviewerDB, PullRequest,
        Repository,
    };
    use github_scbot_redis::{LockInstance, LockStatus};
    use mockall::predicate;

    use crate::commands::CommandContextTest;

    use super::*;

    #[actix_rt::test]
    async fn test_command() -> Result<()> {
        let mut ctx = CommandContextTest::new();
        ctx.db_adapter.expect_repositories().returning(|| {
            let mut mock = MockRepositoryDB::new();
            mock.expect_get()
                .with(predicate::eq("owner"), predicate::eq("name"))
                .returning(|_, _| {
                    async { Ok(Some(Repository::builder().build().unwrap())) }.boxed()
                });

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
                    predicate::eq(0),
                )
                .returning(|_, _, _, _| {
                    async { Ok(PullRequest::builder().build().unwrap()) }.boxed()
                });
            mock.expect_set_status_comment_id()
                .with(
                    predicate::eq("owner"),
                    predicate::eq("name"),
                    predicate::eq(1),
                    predicate::eq(2),
                )
                .returning(|_, _, _, _| {
                    async { Ok(PullRequest::builder().build().unwrap()) }.boxed()
                });

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
        ctx.db_adapter.expect_merge_rules().returning(|| {
            let mut mock = MockMergeRuleDB::new();
            mock.expect_get()
                .returning(|_, _, _, _| async { Ok(None) }.boxed());

            Box::new(mock)
        });
        ctx.api_adapter
            .expect_pull_reviews_list()
            .with(
                predicate::eq("owner"),
                predicate::eq("name"),
                predicate::eq(1),
            )
            .returning(|_, _, _| Ok(vec![]));
        ctx.api_adapter
            .expect_comments_post()
            .returning(|_, _, _, _| Ok(2));
        ctx.redis_adapter
            .expect_wait_lock_resource()
            .returning(|_, _| {
                Ok(LockStatus::SuccessfullyLocked(LockInstance::new_dummy(
                    "pouet",
                )))
            });

        let result = AdminResetSummaryCommand::new()
            .handle(&ctx.as_context())
            .await?;
        assert!(result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![ResultAction::AddReaction(GhReactionType::Eyes)]
        );

        Ok(())
    }
}
