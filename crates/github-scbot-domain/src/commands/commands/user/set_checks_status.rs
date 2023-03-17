use github_scbot_core::types::{issues::GhReactionType, status::CheckStatus};

use async_trait::async_trait;

use crate::{
    commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    Result,
};

pub struct SetChecksStatusCommand {
    status: CheckStatus,
}

impl SetChecksStatusCommand {
    pub fn new_skip_or_wait(status: bool) -> Self {
        Self {
            status: if status {
                CheckStatus::Skipped
            } else {
                CheckStatus::Waiting
            },
        }
    }
}

#[async_trait(?Send)]
impl BotCommand for SetChecksStatusCommand {
    async fn handle(&self, ctx: &mut CommandContext) -> Result<CommandExecutionResult> {
        let value = !matches!(self.status, CheckStatus::Skipped);

        ctx.db_service
            .pull_requests_set_checks_enabled(ctx.repo_owner, ctx.repo_name, ctx.pr_number, value)
            .await?;

        let comment = format!(
            "Check status is marked as **{}** by **{}**.",
            self.status.to_str(),
            ctx.comment_author
        );

        Ok(CommandExecutionResult::builder()
            .with_status_update(true)
            .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
            .with_action(ResultAction::PostComment(comment))
            .build())
    }
}
