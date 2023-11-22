use async_trait::async_trait;
use github_scbot_domain_models::{MergeStrategy, RuleBranch};
use github_scbot_ghapi_interface::types::GhReactionType;

use crate::{
    commands::{BotCommand, CommandContext, CommandExecutionResult, ResultAction},
    use_cases::repositories::{AddMergeRuleUseCase, AddMergeRuleUseCaseInterface},
    Result,
};

pub struct AdminAddMergeRuleCommand {
    base: RuleBranch,
    head: RuleBranch,
    strategy: MergeStrategy,
}

impl AdminAddMergeRuleCommand {
    pub fn new(base: RuleBranch, head: RuleBranch, strategy: MergeStrategy) -> Self {
        Self {
            base,
            head,
            strategy,
        }
    }
}

#[async_trait(?Send)]
impl BotCommand for AdminAddMergeRuleCommand {
    async fn handle(&self, ctx: &CommandContext) -> Result<CommandExecutionResult> {
        let repository = ctx
            .db_service
            .repositories_get(ctx.repo_owner, ctx.repo_name)
            .await?
            .unwrap();

        AddMergeRuleUseCase {
            db_service: ctx.db_service,
        }
        .run(
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
