use async_trait::async_trait;
use prbot_ghapi_interface::types::GhReactionType;

use crate::{
    bot_commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    Result,
};

pub struct SetLabelsCommand {
    added: Vec<String>,
    removed: Vec<String>,
}

impl SetLabelsCommand {
    pub fn new_added(added: Vec<String>) -> Self {
        Self {
            added,
            removed: vec![],
        }
    }

    pub fn new_removed(removed: Vec<String>) -> Self {
        Self {
            added: vec![],
            removed,
        }
    }
}

#[async_trait]
impl BotCommand for SetLabelsCommand {
    async fn handle(&self, ctx: &CommandContext) -> Result<CommandExecutionResult> {
        if !self.added.is_empty() {
            ctx.api_service
                .issue_labels_add(ctx.repo_owner, ctx.repo_name, ctx.pr_number, &self.added)
                .await?;
        }

        if !self.removed.is_empty() {
            ctx.api_service
                .issue_labels_remove(ctx.repo_owner, ctx.repo_name, ctx.pr_number, &self.removed)
                .await?;
        }

        Ok(CommandExecutionResult::builder()
            .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
            .build())
    }
}
