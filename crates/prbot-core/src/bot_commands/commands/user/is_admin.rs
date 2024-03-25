use async_trait::async_trait;
use prbot_ghapi_interface::types::GhReactionType;

use crate::{
    bot_commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    use_cases::auth::CheckIsAdmin,
    Result,
};

pub struct IsAdminCommand;

impl IsAdminCommand {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BotCommand for IsAdminCommand {
    async fn handle(&self, ctx: &CommandContext) -> Result<CommandExecutionResult> {
        let is_admin = CheckIsAdmin
            .run(&ctx.as_core_context(), ctx.comment_author)
            .await?;

        let reaction_type = if is_admin {
            GhReactionType::PlusOne
        } else {
            GhReactionType::MinusOne
        };

        Ok(CommandExecutionResult::builder()
            .with_action(ResultAction::AddReaction(reaction_type))
            .build())
    }
}
