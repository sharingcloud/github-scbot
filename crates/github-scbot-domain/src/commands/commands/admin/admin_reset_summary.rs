use github_scbot_core::types::issues::GhReactionType;

use async_trait::async_trait;

use crate::{
    commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    status::PullRequestStatus,
    summary::SummaryCommentSender,
    Result,
};

pub struct AdminResetSummaryCommand;

impl AdminResetSummaryCommand {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait(?Send)]
impl BotCommand for AdminResetSummaryCommand {
    async fn handle(&self, ctx: &mut CommandContext) -> Result<CommandExecutionResult> {
        let status = PullRequestStatus::from_database(
            ctx.api_adapter,
            ctx.db_adapter,
            ctx.repo_owner,
            ctx.repo_name,
            ctx.pr_number,
            ctx.upstream_pr,
        )
        .await?;

        // Reset comment ID
        ctx.db_adapter
            .pull_requests_set_status_comment_id(ctx.repo_owner, ctx.repo_name, ctx.pr_number, 0)
            .await?;

        SummaryCommentSender::create_or_update(
            ctx.api_adapter,
            ctx.db_adapter,
            ctx.redis_adapter,
            ctx.repo_owner,
            ctx.repo_name,
            ctx.pr_number,
            &status,
        )
        .await?;

        Ok(CommandExecutionResult::builder()
            .with_status_update(true)
            .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
            .build())
    }
}
