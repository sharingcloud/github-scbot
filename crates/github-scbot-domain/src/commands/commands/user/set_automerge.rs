use github_scbot_core::types::issues::GhReactionType;

use async_trait::async_trait;

use crate::{
    commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    Result,
};

pub struct SetAutomergeCommand {
    enabled: bool,
}

impl SetAutomergeCommand {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    pub fn new_enabled() -> Self {
        Self { enabled: true }
    }

    pub fn new_disabled() -> Self {
        Self { enabled: false }
    }
}

#[async_trait(?Send)]
impl BotCommand for SetAutomergeCommand {
    async fn handle(&self, ctx: &mut CommandContext) -> Result<CommandExecutionResult> {
        ctx.db_service
            .pull_requests_set_automerge(ctx.repo_owner, ctx.repo_name, ctx.pr_number, self.enabled)
            .await?;

        let status_text = if self.enabled { "enabled" } else { "disabled" };
        let comment = format!(
            "Automerge **{}** by **{}**.",
            status_text, ctx.comment_author
        );
        Ok(CommandExecutionResult::builder()
            .with_status_update(true)
            .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
            .with_action(ResultAction::PostComment(comment))
            .build())
    }
}
