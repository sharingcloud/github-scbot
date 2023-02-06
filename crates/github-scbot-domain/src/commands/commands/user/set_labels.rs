use github_scbot_core::types::issues::GhReactionType;

use async_trait::async_trait;

use crate::{
    commands::{
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

#[async_trait(?Send)]
impl BotCommand for SetLabelsCommand {
    async fn handle(&self, ctx: &mut CommandContext) -> Result<CommandExecutionResult> {
        if !self.added.is_empty() {
            ctx.api_adapter
                .issue_labels_add(ctx.repo_owner, ctx.repo_name, ctx.pr_number, &self.added)
                .await?;
        }

        if !self.removed.is_empty() {
            ctx.api_adapter
                .issue_labels_remove(ctx.repo_owner, ctx.repo_name, ctx.pr_number, &self.removed)
                .await?;
        }

        Ok(CommandExecutionResult::builder()
            .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
            .build())
    }
}
