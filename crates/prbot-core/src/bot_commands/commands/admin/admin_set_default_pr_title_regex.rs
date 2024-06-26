use async_trait::async_trait;
use prbot_ghapi_interface::types::GhReactionType;

use crate::{
    bot_commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    Result,
};

pub struct AdminSetDefaultPrTitleRegexCommand {
    regex: String,
}

impl AdminSetDefaultPrTitleRegexCommand {
    pub fn new(regex: String) -> Self {
        Self { regex }
    }
}

#[async_trait]
impl BotCommand for AdminSetDefaultPrTitleRegexCommand {
    async fn handle(&self, ctx: &CommandContext) -> Result<CommandExecutionResult> {
        ctx.db_service
            .repositories_set_pr_title_validation_regex(ctx.repo_owner, ctx.repo_name, &self.regex)
            .await?;

        let comment = if self.regex.is_empty() {
            "Pull request title regex **unset** for this repository.".into()
        } else {
            format!(
                "Pull request title regex set to **{}** for this repository.",
                self.regex
            )
        };
        Ok(CommandExecutionResult::builder()
            .with_status_update(true)
            .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
            .with_action(ResultAction::PostComment(comment))
            .build())
    }
}
