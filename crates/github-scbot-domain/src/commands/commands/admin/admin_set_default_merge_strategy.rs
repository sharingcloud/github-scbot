use github_scbot_core::types::{issues::GhReactionType, pulls::GhMergeStrategy};

use async_trait::async_trait;

use crate::{
    commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    Result,
};

pub struct AdminSetDefaultMergeStrategyCommand {
    strategy: GhMergeStrategy,
}

impl AdminSetDefaultMergeStrategyCommand {
    pub fn new(strategy: GhMergeStrategy) -> Self {
        Self { strategy }
    }
}

#[async_trait(?Send)]
impl BotCommand for AdminSetDefaultMergeStrategyCommand {
    async fn handle(&self, ctx: &mut CommandContext) -> Result<CommandExecutionResult> {
        ctx.db_service
            .repositories_set_default_strategy(ctx.repo_owner, ctx.repo_name, self.strategy)
            .await?;

        let comment = format!(
            "Merge strategy set to **{}** for this repository.",
            self.strategy
        );
        Ok(CommandExecutionResult::builder()
            .with_status_update(false)
            .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
            .with_action(ResultAction::PostComment(comment))
            .build())
    }
}
