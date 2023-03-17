use async_trait::async_trait;
use github_scbot_ghapi_interface::types::GhReactionType;

use crate::{
    commands::{
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

#[async_trait(?Send)]
impl BotCommand for PingCommand {
    async fn handle(&self, ctx: &mut CommandContext) -> Result<CommandExecutionResult> {
        let comment = format!("**{}** pong!", ctx.comment_author);
        Ok(CommandExecutionResult::builder()
            .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
            .with_action(ResultAction::PostComment(comment))
            .build())
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_ghapi_interface::types::GhReactionType;

    use super::*;
    use crate::commands::CommandContextTest;

    #[actix_rt::test]
    async fn test_command() -> Result<()> {
        let mut ctx = CommandContextTest::new();
        let cmd = PingCommand::new();

        let result = cmd.handle(&mut ctx.as_context()).await?;
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
