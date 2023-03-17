use async_trait::async_trait;
use github_scbot_core::types::issues::GhReactionType;

use crate::{
    commands::{
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

#[async_trait(?Send)]
impl BotCommand for AdminSetDefaultPrTitleRegexCommand {
    async fn handle(&self, ctx: &mut CommandContext) -> Result<CommandExecutionResult> {
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
