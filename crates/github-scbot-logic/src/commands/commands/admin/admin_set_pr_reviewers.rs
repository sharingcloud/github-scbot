use github_scbot_core::types::issues::GhReactionType;

use async_trait::async_trait;

use crate::{
    commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    Result,
};

pub struct AdminSetPrReviewersCommand {
    count: u64,
}

impl AdminSetPrReviewersCommand {
    pub fn new(count: u64) -> Self {
        Self { count }
    }
}

#[async_trait(?Send)]
impl BotCommand for AdminSetPrReviewersCommand {
    async fn handle(&self, ctx: &CommandContext) -> Result<CommandExecutionResult> {
        ctx.db_adapter
            .pull_requests()
            .set_needed_reviewers_count(ctx.repo_owner, ctx.repo_name, ctx.pr_number, self.count)
            .await?;

        let comment = format!(
            "Needed reviewers count set to **{}** for this pull request.",
            self.count
        );
        Ok(CommandExecutionResult::builder()
            .with_status_update(true)
            .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
            .with_action(ResultAction::PostComment(comment))
            .build())
    }
}

#[cfg(test)]
mod tests {
    use futures_util::FutureExt;
    use github_scbot_database::{MockPullRequestDB, PullRequest};
    use mockall::predicate;

    use crate::commands::CommandContextTest;

    use super::*;

    #[actix_rt::test]
    async fn test_enable() -> Result<()> {
        let mut ctx = CommandContextTest::new();
        ctx.db_adapter.expect_pull_requests().returning(|| {
            let mut mock = MockPullRequestDB::new();
            mock.expect_set_needed_reviewers_count()
                .with(
                    predicate::eq("owner"),
                    predicate::eq("name"),
                    predicate::eq(1),
                    predicate::eq(0),
                )
                .returning(|_, _, _, _| {
                    async { Ok(PullRequest::builder().build().unwrap()) }.boxed()
                });

            Box::new(mock)
        });

        let result = AdminSetPrReviewersCommand::new(0)
            .handle(&ctx.as_context())
            .await?;
        assert!(result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![
                ResultAction::AddReaction(GhReactionType::Eyes),
                ResultAction::PostComment(
                    "Needed reviewers count set to **0** for this pull request.".into()
                )
            ]
        );

        Ok(())
    }
}
