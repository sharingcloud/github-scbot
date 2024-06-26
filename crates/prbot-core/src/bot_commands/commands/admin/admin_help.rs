use async_trait::async_trait;
use prbot_ghapi_interface::types::GhReactionType;

use crate::{
    bot_commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    Result,
};

pub struct AdminHelpCommand;

impl AdminHelpCommand {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BotCommand for AdminHelpCommand {
    async fn handle(&self, ctx: &CommandContext) -> Result<CommandExecutionResult> {
        let comment = format!(
            "Hello **{}** ! I am a GitHub helper bot ! :robot:\n\
            You can ping me with a command in the format: `{} <command> (<arguments>)`\n\
            \n\
            Supported admin commands:\n\
            - `admin-help`: _Show this comment_\n\
            - `admin-enable`: _Enable me on a pull request with manual interaction_\n\
            - `admin-disable`: _Disable me on a pull request with manual interaction_\n\
            - `admin-add-merge-rule <base> <head> <strategy>`: _Add/Update a merge rule for this repository_\n\
            - `admin-set-default-needed-reviewers <count>`: _Set default needed reviewers count for this repository_\n\
            - `admin-set-default-merge-strategy <merge|squash|rebase>`: _Set default merge strategy for this repository_\n\
            - `admin-set-default-pr-title-regex <regex?>`: _Set default PR title validation regex for this repository_\n\
            - `admin-set-default-automerge+`: _Set automerge enabled for this repository_\n\
            - `admin-set-default-automerge-`: _Set automerge disabled for this repository_\n\
            - `admin-set-default-qa-status+`: _Enable QA validation by default for this repository_\n\
            - `admin-set-default-qa-status-`: _Disable QA validation by default for this repository_\n\
            - `admin-set-default-checks-status+`: _Enable checks validation by default for this repository_\n\
            - `admin-set-default-checks-status-`: _Disable checks validation by default for this repository_\n\
            - `admin-set-needed-reviewers <count>`: _Set needed reviewers count for this PR_\n\
            - `admin-reset-reviewers`: _Reset and update reviews on pull request (maintenance-type command)_\n\
            - `admin-reset-summary`: _Create a new summary message (maintenance-type command)_\n\
            - `admin-sync`: _Update status comment if needed (maintenance-type command)_\n",
            ctx.comment_author, ctx.config.name
        );

        Ok(CommandExecutionResult::builder()
            .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
            .with_action(ResultAction::PostComment(comment))
            .build())
    }
}
