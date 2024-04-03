use async_trait::async_trait;
use prbot_ghapi_interface::types::GhReactionType;
use shaku::HasComponent;

use crate::{
    bot_commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    use_cases::{status::BuildPullRequestStatusInterface, summary::PostSummaryCommentInterface},
    Result,
};

pub struct AdminResetSummaryCommand;

impl AdminResetSummaryCommand {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BotCommand for AdminResetSummaryCommand {
    async fn handle(&self, ctx: &CommandContext) -> Result<CommandExecutionResult> {
        let build_pr_status: &dyn BuildPullRequestStatusInterface = ctx.core_module.resolve_ref();
        let pr_status = build_pr_status
            .run(&ctx.as_core_context(), &ctx.pr_handle(), ctx.upstream_pr)
            .await?;

        // Reset comment ID
        ctx.db_service
            .pull_requests_set_status_comment_id(ctx.repo_owner, ctx.repo_name, ctx.pr_number, 0)
            .await?;

        let post_summary_comment: &dyn PostSummaryCommentInterface = ctx.core_module.resolve_ref();
        post_summary_comment
            .run(&ctx.as_core_context(), &ctx.pr_handle(), &pr_status)
            .await?;

        Ok(CommandExecutionResult::builder()
            .with_status_update(true)
            .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
            .build())
    }
}
