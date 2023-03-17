use github_scbot_core::types::issues::GhReactionType;

use async_trait::async_trait;

use crate::{
    commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    Result,
};

pub struct AdminSetDefaultReviewersCommand {
    count: u64,
}

impl AdminSetDefaultReviewersCommand {
    pub fn new(count: u64) -> Self {
        Self { count }
    }
}

#[async_trait(?Send)]
impl BotCommand for AdminSetDefaultReviewersCommand {
    async fn handle(&self, ctx: &mut CommandContext) -> Result<CommandExecutionResult> {
        ctx.db_service
            .repositories_set_default_needed_reviewers_count(
                ctx.repo_owner,
                ctx.repo_name,
                self.count,
            )
            .await?;

        let comment = format!(
            "Needed reviewers count set to **{}** for this repository.",
            self.count
        );
        Ok(CommandExecutionResult::builder()
            .with_status_update(false)
            .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
            .with_action(ResultAction::PostComment(comment))
            .build())
    }
}
