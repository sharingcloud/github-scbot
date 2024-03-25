use async_trait::async_trait;
use prbot_ghapi_interface::types::GhReactionType;

use crate::{
    bot_commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    Result,
};

pub struct PingCommand;

impl PingCommand {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BotCommand for PingCommand {
    async fn handle(&self, ctx: &CommandContext) -> Result<CommandExecutionResult> {
        let comment = format!("**{}** pong!", ctx.comment_author);
        Ok(CommandExecutionResult::builder()
            .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
            .with_action(ResultAction::PostComment(comment))
            .build())
    }
}

#[cfg(test)]
mod tests {
    use prbot_ghapi_interface::types::GhReactionType;

    use super::*;
    use crate::bot_commands::CommandContextTest;

    #[tokio::test]
    async fn test_command() -> Result<()> {
        let ctx = CommandContextTest::new();
        let cmd = PingCommand::new();

        let result = cmd.handle(&ctx.as_context()).await?;
        assert!(!result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![
                ResultAction::AddReaction(GhReactionType::Eyes),
                ResultAction::PostComment("**me** pong!".into())
            ]
        );

        Ok(())
    }
}
