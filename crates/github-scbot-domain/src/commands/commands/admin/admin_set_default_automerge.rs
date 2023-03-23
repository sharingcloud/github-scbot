use async_trait::async_trait;
use github_scbot_ghapi_interface::types::GhReactionType;

use crate::{
    commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    Result,
};

pub struct AdminSetDefaultAutomergeCommand {
    enabled: bool,
}

impl AdminSetDefaultAutomergeCommand {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }
}

#[async_trait(?Send)]
impl BotCommand for AdminSetDefaultAutomergeCommand {
    async fn handle(&self, ctx: &CommandContext) -> Result<CommandExecutionResult> {
        ctx.db_service
            .repositories_set_default_automerge(ctx.repo_owner, ctx.repo_name, self.enabled)
            .await?;

        let comment = format!(
            "Default automerge status set to **{}** for this repository.",
            self.enabled
        );
        Ok(CommandExecutionResult::builder()
            .with_status_update(true)
            .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
            .with_action(ResultAction::PostComment(comment))
            .build())
    }
}
