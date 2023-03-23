use async_trait::async_trait;
use github_scbot_ghapi_interface::types::GhReactionType;

use crate::{
    commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    Result,
};

pub struct AdminSetDefaultChecksStatusCommand {
    enabled: bool,
}

impl AdminSetDefaultChecksStatusCommand {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }
}

#[async_trait(?Send)]
impl BotCommand for AdminSetDefaultChecksStatusCommand {
    async fn handle(&self, ctx: &CommandContext) -> Result<CommandExecutionResult> {
        ctx.db_service
            .repositories_set_default_enable_checks(ctx.repo_owner, ctx.repo_name, self.enabled)
            .await?;

        let comment = if self.enabled {
            "Checks **enabled** for this repository."
        } else {
            "Checks **disabled** for this repository."
        };
        Ok(CommandExecutionResult::builder()
            .with_status_update(true)
            .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
            .with_action(ResultAction::PostComment(comment.into()))
            .build())
    }
}
