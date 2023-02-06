use github_scbot_core::types::issues::GhReactionType;

use async_trait::async_trait;

use crate::{
    commands::{
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

#[async_trait(?Send)]
impl BotCommand for LockCommand {
    async fn handle(&self, ctx: &mut CommandContext) -> Result<CommandExecutionResult> {
        let status_text = if self.locked { "locked" } else { "unlocked" };
        ctx.db_adapter
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
