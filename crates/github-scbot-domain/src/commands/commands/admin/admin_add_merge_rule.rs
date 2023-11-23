use async_trait::async_trait;
use github_scbot_domain_models::{MergeStrategy, RuleBranch};
use github_scbot_ghapi_interface::types::GhReactionType;

use crate::{
    commands::{BotCommand, CommandContext, CommandExecutionResult, ResultAction},
    use_cases::repositories::{AddMergeRuleUseCase, AddMergeRuleUseCaseInterface},
    Result,
};

pub struct AdminAddMergeRuleCommand<'a> {
    pub base: RuleBranch,
    pub head: RuleBranch,
    pub strategy: MergeStrategy,
    pub add_merge_rule_uc: &'a dyn AddMergeRuleUseCaseInterface,
}

#[async_trait(?Send)]
impl<'a> BotCommand for AdminAddMergeRuleCommand<'a> {
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

#[cfg(test)]
mod tests {
    use std::error::Error;

    use async_trait::async_trait;
    use github_scbot_database_interface::DbService;
    use github_scbot_domain_models::{MergeStrategy, Repository, RuleBranch};

    use super::AdminAddMergeRuleCommand;
    use crate::{
        commands::{BotCommand, CommandContextTest},
        use_cases::repositories::AddMergeRuleUseCaseInterface,
    };

    #[tokio::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let repo_name: String = "foo".into();
        let repo_owner: String = "bar".into();

        let mut ctx = CommandContextTest::new();
        ctx.repo_name = repo_name.clone();
        ctx.repo_owner = repo_owner.clone();
        ctx.db_service
            .repositories_create(Repository {
                owner: repo_owner.clone(),
                name: repo_name.clone(),
                ..Default::default()
            })
            .await?;

        pub struct Dummy;

        #[async_trait(?Send)]
        impl AddMergeRuleUseCaseInterface for Dummy {
            async fn run(
                &self,
                _repository: &Repository,
                _base: RuleBranch,
                _head: RuleBranch,
                _strategy: MergeStrategy,
            ) -> crate::Result<()> {
                Ok(())
            }
        }

        let cmd = AdminAddMergeRuleCommand {
            base: RuleBranch::Wildcard,
            head: RuleBranch::Wildcard,
            strategy: MergeStrategy::Squash,
            add_merge_rule_uc: &Dummy,
        };

        let result = cmd.handle(&ctx.as_context()).await?;
        assert!(result.should_update_status);

        Ok(())
    }
}
