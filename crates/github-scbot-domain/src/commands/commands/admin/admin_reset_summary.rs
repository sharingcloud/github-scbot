use async_trait::async_trait;
use github_scbot_ghapi_interface::types::GhReactionType;

use crate::{
    commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    use_cases::{status::BuildPullRequestStatusUseCase, summary::PostSummaryCommentUseCase},
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
        let pr_status = BuildPullRequestStatusUseCase {
            api_service: ctx.api_service,
            db_service: ctx.db_service,
            repo_owner: ctx.repo_owner,
            repo_name: ctx.repo_name,
            pr_number: ctx.pr_number,
            upstream_pr: ctx.upstream_pr,
        }
        .run()
        .await?;

        // Reset comment ID
        ctx.db_service
            .pull_requests_set_status_comment_id(ctx.repo_owner, ctx.repo_name, ctx.pr_number, 0)
            .await?;

        PostSummaryCommentUseCase {
            api_service: ctx.api_service,
            db_service: ctx.db_service,
            lock_service: ctx.lock_service,
            repo_owner: ctx.repo_owner,
            repo_name: ctx.repo_name,
            pr_number: ctx.pr_number,
            pr_status: &pr_status,
        }
        .run()
        .await?;

        Ok(CommandExecutionResult::builder()
            .with_status_update(true)
            .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
            .build())
    }
}
