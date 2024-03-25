use async_trait::async_trait;
use prbot_ghapi_interface::types::GhReactionType;
use shaku::HasComponent;

use crate::{
    bot_commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    use_cases::status::DisablePullRequestStatusInterface,
    Result,
};

pub struct AdminDisableCommand;

impl AdminDisableCommand {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BotCommand for AdminDisableCommand {
    async fn handle(&self, ctx: &CommandContext) -> Result<CommandExecutionResult> {
        let repo_model = ctx
            .db_service
            .repositories_get(ctx.repo_owner, ctx.repo_name)
            .await?
            .unwrap();

        if repo_model.manual_interaction {
            let disable_pr_status: &dyn DisablePullRequestStatusInterface =
                ctx.core_module.resolve_ref();
            disable_pr_status
                .run(&ctx.as_core_context(), &ctx.pr_handle())
                .await?;

            ctx.db_service
                .pull_requests_delete(ctx.repo_owner, ctx.repo_name, ctx.pr_number)
                .await?;

            let comment = "Bot disabled on this PR. Bye!";
            Ok(CommandExecutionResult::builder()
                .with_status_update(false)
                .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
                .with_action(ResultAction::PostComment(comment.into()))
                .build())
        } else {
            let comment = "You can not disable the bot on this PR, the repository is not in manual interaction mode.";
            Ok(CommandExecutionResult::builder()
                .denied()
                .with_status_update(false)
                .with_action(ResultAction::AddReaction(GhReactionType::MinusOne))
                .with_action(ResultAction::PostComment(comment.into()))
                .build())
        }
    }
}
