use github_scbot_core::types::issues::GhReactionType;

use async_trait::async_trait;

use crate::{
    commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    Result,
};

pub struct AdminSetPrReviewersCommand {
    count: u64,
}

impl AdminSetPrReviewersCommand {
    pub fn new(count: u64) -> Self {
        Self { count }
    }
}

#[async_trait(?Send)]
impl BotCommand for AdminSetPrReviewersCommand {
    async fn handle(&self, ctx: &mut CommandContext) -> Result<CommandExecutionResult> {
        ctx.db_service
            .pull_requests_set_needed_reviewers_count(
                ctx.repo_owner,
                ctx.repo_name,
                ctx.pr_number,
                self.count,
            )
            .await?;

        let comment = format!(
            "Needed reviewers count set to **{}** for this pull request.",
            self.count
        );
        Ok(CommandExecutionResult::builder()
            .with_status_update(true)
            .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
            .with_action(ResultAction::PostComment(comment))
            .build())
    }
}
