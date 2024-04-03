use async_trait::async_trait;
use prbot_ghapi_interface::types::GhReactionType;

use crate::{
    bot_commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    Result,
};

pub struct LockCommand {
    locked: bool,
    reason: Option<String>,
}

impl LockCommand {
    pub fn new(locked: bool, reason: Option<String>) -> Self {
        Self { locked, reason }
    }
}

#[async_trait]
impl BotCommand for LockCommand {
    async fn handle(&self, ctx: &CommandContext) -> Result<CommandExecutionResult> {
        let status_text = if self.locked { "locked" } else { "unlocked" };
        ctx.db_service
            .pull_requests_set_locked(ctx.repo_owner, ctx.repo_name, ctx.pr_number, self.locked)
            .await?;

        let mut comment = format!(
            "Pull request **{}** by **{}**.",
            status_text, ctx.comment_author
        );
        if let Some(reason) = &self.reason {
            comment = format!("{}\n**Reason**: {}.", comment, reason);
        }

        Ok(CommandExecutionResult::builder()
            .with_status_update(true)
            .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
            .with_action(ResultAction::PostComment(comment))
            .build())
    }
}
