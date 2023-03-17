use async_trait::async_trait;
use github_scbot_core::types::{issues::GhReactionType, pulls::GhMergeStrategy};

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
        ctx: &mut CommandContext<'a>,
        strategy: GhMergeStrategy,
    ) -> Result<CommandExecutionResult> {
        ctx.db_service
            .pull_requests_set_strategy_override(
                ctx.repo_owner,
                ctx.repo_name,
                ctx.pr_number,
                Some(strategy),
            )
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
        ctx: &mut CommandContext<'a>,
    ) -> Result<CommandExecutionResult> {
        ctx.db_service
            .pull_requests_set_strategy_override(ctx.repo_owner, ctx.repo_name, ctx.pr_number, None)
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
    async fn handle(&self, ctx: &mut CommandContext) -> Result<CommandExecutionResult> {
        if let Some(strategy) = self.strategy {
            self._handle_set_strategy(ctx, strategy).await
        } else {
            self._handle_unset_strategy(ctx).await
        }
    }
}
