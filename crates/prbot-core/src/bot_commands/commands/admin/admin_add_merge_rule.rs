use async_trait::async_trait;
use prbot_ghapi_interface::types::GhReactionType;
use prbot_models::{MergeStrategy, RuleBranch};
use shaku::HasComponent;

use crate::{
    bot_commands::{BotCommand, CommandContext, CommandExecutionResult, ResultAction},
    use_cases::repositories::AddMergeRuleInterface,
    Result,
};

pub struct AdminAddMergeRuleCommand {
    pub base: RuleBranch,
    pub head: RuleBranch,
    pub strategy: MergeStrategy,
}

#[async_trait]
impl BotCommand for AdminAddMergeRuleCommand {
    async fn handle(&self, ctx: &CommandContext) -> Result<CommandExecutionResult> {
        let repository = ctx
            .db_service
            .repositories_get(ctx.repo_owner, ctx.repo_name)
            .await?
            .unwrap();

        let uc: &dyn AddMergeRuleInterface = ctx.core_module.resolve_ref();
        uc.run(
            &ctx.as_core_context(),
            &repository,
            self.base.clone(),
            self.head.clone(),
            self.strategy,
        )
        .await?;

        let comment = if self.base == RuleBranch::Wildcard && self.head == RuleBranch::Wildcard {
            format!(
                "Default strategy updated to '{}' for repository '{}'",
                self.strategy,
                ctx.repository_path()
            )
        } else {
            format!("Merge rule created/updated with '{}' for repository '{}' and branches '{}' (base) <- '{}' (head)", self.strategy, ctx.repository_path(), self.base, self.head)
        };

        Ok(CommandExecutionResult::builder()
            .with_status_update(true)
            .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
            .with_action(ResultAction::PostComment(comment))
            .build())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use prbot_database_interface::DbService;
    use prbot_models::{MergeStrategy, Repository, RuleBranch};

    use super::AdminAddMergeRuleCommand;
    use crate::bot_commands::{BotCommand, CommandContextTest};

    #[tokio::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let repo_name: String = "foo".into();
        let repo_owner: String = "bar".into();

        let mut ctx = CommandContextTest::new();
        ctx.repo_name.clone_from(&repo_name);
        ctx.repo_owner.clone_from(&repo_owner);
        ctx.db_service
            .repositories_create(Repository {
                owner: repo_owner.clone(),
                name: repo_name.clone(),
                ..Default::default()
            })
            .await?;

        let cmd = AdminAddMergeRuleCommand {
            base: RuleBranch::Wildcard,
            head: RuleBranch::Wildcard,
            strategy: MergeStrategy::Squash,
        };

        let result = cmd.handle(&ctx.as_context()).await?;
        assert!(result.should_update_status);

        Ok(())
    }
}
