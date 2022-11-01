use github_scbot_core::types::{issues::GhReactionType, pulls::GhMergeStrategy};

use async_trait::async_trait;

use crate::{
    commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    Result,
};

pub struct SetMergeStrategyCommand {
    strategy: Option<GhMergeStrategy>,
}

impl SetMergeStrategyCommand {
    pub fn new(status: GhMergeStrategy) -> Self {
        Self {
            strategy: Some(status),
        }
    }

    pub fn new_unset() -> Self {
        Self { strategy: None }
    }

    async fn _handle_set_strategy<'a>(
        &self,
        ctx: &CommandContext<'a>,
        strategy: GhMergeStrategy,
    ) -> Result<CommandExecutionResult> {
        ctx.db_adapter
            .pull_requests()
            .set_strategy_override(ctx.repo_owner, ctx.repo_name, ctx.pr_number, Some(strategy))
            .await?;

        let comment = format!(
            "Merge strategy set to **{}** for this pull request.",
            strategy
        );

        Ok(CommandExecutionResult::builder()
            .with_status_update(true)
            .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
            .with_action(ResultAction::PostComment(comment))
            .build())
    }

    async fn _handle_unset_strategy<'a>(
        &self,
        ctx: &CommandContext<'a>,
    ) -> Result<CommandExecutionResult> {
        ctx.db_adapter
            .pull_requests()
            .set_strategy_override(ctx.repo_owner, ctx.repo_name, ctx.pr_number, None)
            .await?;

        let comment = "Merge strategy override removed for this pull request.".into();
        Ok(CommandExecutionResult::builder()
            .with_status_update(true)
            .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
            .with_action(ResultAction::PostComment(comment))
            .build())
    }
}

#[async_trait(?Send)]
impl BotCommand for SetMergeStrategyCommand {
    async fn handle(&self, ctx: &CommandContext) -> Result<CommandExecutionResult> {
        if let Some(strategy) = self.strategy {
            self._handle_set_strategy(ctx, strategy).await
        } else {
            self._handle_unset_strategy(ctx).await
        }
    }
}

#[cfg(test)]
mod tests {
    use futures_util::FutureExt;
    use github_scbot_database::MockPullRequestDB;
    use github_scbot_database::PullRequest;
    use mockall::predicate;

    use crate::commands::CommandContextTest;

    use super::*;

    #[actix_rt::test]
    async fn test_set() -> Result<()> {
        let mut ctx = CommandContextTest::new();
        ctx.db_adapter
            .expect_pull_requests()
            .times(1)
            .returning(|| {
                let mut mock = MockPullRequestDB::new();
                mock.expect_set_strategy_override()
                    .times(1)
                    .with(
                        predicate::eq("owner"),
                        predicate::eq("name"),
                        predicate::eq(1),
                        predicate::eq(Some(GhMergeStrategy::Merge)),
                    )
                    .returning(|_, _, _, _| {
                        async { Ok(PullRequest::builder().build().unwrap()) }.boxed()
                    });

                Box::new(mock)
            });

        let cmd = SetMergeStrategyCommand::new(GhMergeStrategy::Merge);
        let result = cmd.handle(&ctx.as_context()).await?;
        assert!(result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![
                ResultAction::AddReaction(GhReactionType::Eyes),
                ResultAction::PostComment(
                    "Merge strategy set to **merge** for this pull request.".into()
                )
            ]
        );

        Ok(())
    }

    #[actix_rt::test]
    async fn test_unset() -> Result<()> {
        let mut ctx = CommandContextTest::new();
        ctx.db_adapter
            .expect_pull_requests()
            .times(1)
            .returning(|| {
                let mut mock = MockPullRequestDB::new();
                mock.expect_set_strategy_override()
                    .times(1)
                    .with(
                        predicate::eq("owner"),
                        predicate::eq("name"),
                        predicate::eq(1),
                        predicate::eq(None),
                    )
                    .returning(|_, _, _, _| {
                        async { Ok(PullRequest::builder().build().unwrap()) }.boxed()
                    });

                Box::new(mock)
            });

        let cmd = SetMergeStrategyCommand::new_unset();
        let result = cmd.handle(&ctx.as_context()).await?;
        assert!(result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![
                ResultAction::AddReaction(GhReactionType::Eyes),
                ResultAction::PostComment(
                    "Merge strategy override removed for this pull request.".into()
                )
            ]
        );

        Ok(())
    }
}
