use async_trait::async_trait;
use github_scbot_ghapi_interface::types::GhReactionType;

use crate::{
    commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    Result,
};

pub struct HelpCommand;

impl HelpCommand {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait(?Send)]
impl BotCommand for HelpCommand {
    async fn handle(&self, ctx: &CommandContext) -> Result<CommandExecutionResult> {
        let comment = format!(
            "Hello **{}** ! I am a GitHub helper bot ! :robot:\n\
            You can ping me with a command in the format: `{} <command> (<arguments>)`\n\
            \n\
            Supported commands:\n\
            - `noqa+`: _Skip QA validation_\n\
            - `noqa-`: _Enable QA validation_\n\
            - `qa+`: _Mark QA as passed_\n\
            - `qa-`: _Mark QA as failed_\n\
            - `qa?`: _Mark QA as waiting_\n\
            - `nochecks+`: _Skip checks validation_\n\
            - `nochecks-`: _Enable checks validation_\n\
            - `automerge+`: _Enable auto-merge for this PR (once all checks pass)_\n\
            - `automerge-`: _Disable auto-merge for this PR_\n\
            - `lock+ <reason?>`: _Lock a pull-request (block merge)_\n\
            - `lock- <reason?>`: _Unlock a pull-request (unblock merge)_\n\
            - `r+ <reviewers>`: _Assign reviewers (you can assign multiple reviewers)_\n\
            - `req+ <reviewers>`: _Assign required reviewers (you can assign multiple reviewers)_\n\
            - `r- <reviewers>`: _Unassign reviewers (you can unassign multiple reviewers)_\n\
            - `strategy+ <strategy>`: _Override merge strategy for this pull request_\n\
            - `strategy-`: _Remove the overriden merge strategy for this pull request_\n\
            - `merge <merge|squash|rebase?>`: _Try merging the pull request with optional strategy_\n\
            - `labels+ <label>`: _Set specific labels_\n\
            - `labels- <label>`: _Unset specific labels_\n\
            - `ping`: _Ping me_\n\
            - `gif <search>`: _Post a random GIF with a tag_\n\
            - `is-admin`: _Check if you are admin_\n\
            - `help`: _Show this comment_\n",
            ctx.comment_author, ctx.config.bot_username
        );

        Ok(CommandExecutionResult::builder()
            .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
            .with_action(ResultAction::PostComment(comment))
            .build())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::CommandContextTest;

    #[tokio::test]
    async fn test_command() -> Result<()> {
        let ctx = CommandContextTest::new();
        let cmd = HelpCommand::new();

        let result = cmd.handle(&ctx.as_context()).await?;
        assert!(!result.should_update_status);

        Ok(())
    }
}
