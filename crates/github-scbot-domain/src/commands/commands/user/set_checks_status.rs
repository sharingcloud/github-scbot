use async_trait::async_trait;
use github_scbot_domain_models::ChecksStatus;
use github_scbot_ghapi_interface::types::GhReactionType;

use crate::{
    commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    Result,
};

pub struct SetChecksStatusCommand {
    status: ChecksStatus,
}

impl SetChecksStatusCommand {
    pub fn new_skip_or_wait(status: bool) -> Self {
        Self {
            status: if status {
                ChecksStatus::Skipped
            } else {
                ChecksStatus::Waiting
            },
        }
    }
}

#[async_trait(?Send)]
impl BotCommand for SetChecksStatusCommand {
    async fn handle(&self, ctx: &mut CommandContext) -> Result<CommandExecutionResult> {
        let value = !matches!(self.status, ChecksStatus::Skipped);

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
