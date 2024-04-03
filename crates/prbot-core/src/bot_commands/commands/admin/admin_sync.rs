use async_trait::async_trait;
use prbot_ghapi_interface::types::GhReactionType;
use shaku::HasComponent;

use crate::{
    bot_commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    use_cases::pulls::SynchronizePullRequestInterface,
    Result,
};

pub struct AdminSyncCommand;

impl AdminSyncCommand {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BotCommand for AdminSyncCommand {
    async fn handle(&self, ctx: &CommandContext) -> Result<CommandExecutionResult> {
        let synchronize_pull_request: &dyn SynchronizePullRequestInterface =
            ctx.core_module.resolve_ref();
        synchronize_pull_request
            .run(&ctx.as_core_context(), &ctx.pr_handle())
            .await?;

        Ok(CommandExecutionResult::builder()
            .with_status_update(true)
            .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
            .build())
    }
}
