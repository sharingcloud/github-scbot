use github_scbot_core::types::issues::GhReactionType;

use async_trait::async_trait;

use crate::{
    commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    status::StatusLogic,
    Result,
};

pub struct AdminDisableCommand;

impl AdminDisableCommand {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait(?Send)]
impl BotCommand for AdminDisableCommand {
    async fn handle(&self, ctx: &mut CommandContext) -> Result<CommandExecutionResult> {
        let repo_model = ctx
            .db_adapter
            .repositories_get(ctx.repo_owner, ctx.repo_name)
            .await?
            .unwrap();

        if repo_model.manual_interaction() {
            StatusLogic::disable_validation_status(
                ctx.api_adapter,
                ctx.db_adapter,
                ctx.repo_owner,
                ctx.repo_name,
                ctx.pr_number,
            )
            .await?;

            ctx.db_adapter
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
