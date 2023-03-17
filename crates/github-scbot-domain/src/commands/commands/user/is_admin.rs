use async_trait::async_trait;
use github_scbot_core::types::issues::GhReactionType;

use crate::{
    commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    use_cases::auth::CheckIsAdminUseCase,
    Result,
};

pub struct IsAdminCommand;

impl IsAdminCommand {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait(?Send)]
impl BotCommand for IsAdminCommand {
    async fn handle(&self, ctx: &mut CommandContext) -> Result<CommandExecutionResult> {
        let is_admin = CheckIsAdminUseCase {
            username: ctx.comment_author,
            db_service: ctx.db_service,
        }
        .run()
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
