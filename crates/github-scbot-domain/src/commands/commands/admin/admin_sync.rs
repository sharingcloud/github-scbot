use github_scbot_core::types::issues::GhReactionType;

use async_trait::async_trait;

use crate::{
    commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    use_cases::pulls::SynchronizePullRequestUseCase,
    Result,
};

pub struct AdminSyncCommand;

impl AdminSyncCommand {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait(?Send)]
impl BotCommand for AdminSyncCommand {
    async fn handle(&self, ctx: &mut CommandContext) -> Result<CommandExecutionResult> {
        SynchronizePullRequestUseCase {
            config: ctx.config,
            db_service: ctx.db_adapter,
            repo_owner: ctx.repo_owner,
            repo_name: ctx.repo_name,
            pr_number: ctx.pr_number,
        }
        .run()
        .await?;

        Ok(CommandExecutionResult::builder()
            .with_status_update(true)
            .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
            .build())
    }
}
