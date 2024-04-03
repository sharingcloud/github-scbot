use async_trait::async_trait;
use prbot_ghapi_interface::types::GhReactionType;
use prbot_models::MergeStrategy;

use crate::{
    bot_commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    Result,
};

pub struct AdminSetDefaultMergeStrategyCommand {
    strategy: MergeStrategy,
}

impl AdminSetDefaultMergeStrategyCommand {
    pub fn new(strategy: MergeStrategy) -> Self {
        Self { strategy }
    }
}

#[async_trait]
impl BotCommand for AdminSetDefaultMergeStrategyCommand {
    async fn handle(&self, ctx: &CommandContext) -> Result<CommandExecutionResult> {
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
