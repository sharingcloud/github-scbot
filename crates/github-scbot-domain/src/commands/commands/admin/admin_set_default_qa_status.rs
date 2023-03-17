use github_scbot_core::types::issues::GhReactionType;

use async_trait::async_trait;

use crate::{
    commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    Result,
};

pub struct AdminSetDefaultQaStatusCommand {
    enabled: bool,
}

impl AdminSetDefaultQaStatusCommand {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }
}

#[async_trait(?Send)]
impl BotCommand for AdminSetDefaultQaStatusCommand {
    async fn handle(&self, ctx: &mut CommandContext) -> Result<CommandExecutionResult> {
        ctx.db_service
            .repositories_set_default_enable_qa(ctx.repo_owner, ctx.repo_name, self.enabled)
            .await?;

        let comment = if self.enabled {
            "QA status check **enabled** for this repository."
        } else {
            "QA status check **disabled** for this repository."
        };
        Ok(CommandExecutionResult::builder()
            .with_status_update(true)
            .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
            .with_action(ResultAction::PostComment(comment.into()))
            .build())
    }
}
