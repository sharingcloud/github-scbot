use async_trait::async_trait;
use prbot_models::{MergeRule, MergeStrategy, Repository, RuleBranch};
use shaku::{Component, Interface};

use crate::{CoreContext, Result};

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait]
pub trait AddMergeRuleInterface: Interface {
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        repository: &'a Repository,
        base: RuleBranch,
        head: RuleBranch,
        strategy: MergeStrategy,
    ) -> Result<()>;
}

#[derive(Component)]
#[shaku(interface = AddMergeRuleInterface)]
pub(crate) struct AddMergeRule;

#[async_trait]
impl AddMergeRuleInterface for AddMergeRule {
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        repository: &'a Repository,
        base: RuleBranch,
        head: RuleBranch,
        strategy: MergeStrategy,
    ) -> Result<()> {
        let owner = &repository.owner;
        let name = &repository.name;

        if base == RuleBranch::Wildcard && head == RuleBranch::Wildcard {
            ctx.db_service
                .repositories_set_default_strategy(owner, name, strategy)
                .await?;
        } else {
            ctx.db_service
                .merge_rules_delete(owner, name, base.clone(), head.clone())
                .await?;
            ctx.db_service
                .merge_rules_create(MergeRule {
                    repository_id: repository.id,
                    base_branch: base,
                    head_branch: head,
                    strategy,
                })
                .await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use prbot_database_interface::DbService;
    use prbot_models::{MergeStrategy, Repository, RuleBranch};

    use super::{AddMergeRule, AddMergeRuleInterface};
    use crate::context::tests::CoreContextTest;

    #[tokio::test]
    async fn run_default() -> Result<(), Box<dyn Error>> {
        let ctx = CoreContextTest::new();
        let repo = ctx
            .db_service
            .repositories_create(Repository {
                default_strategy: MergeStrategy::Merge,
                ..Default::default()
            })
            .await?;

        AddMergeRule
            .run(
                &ctx.as_context(),
                &repo,
                RuleBranch::Wildcard,
                RuleBranch::Wildcard,
                MergeStrategy::Squash,
            )
            .await?;

        let repo = ctx
            .db_service
            .repositories_get(&repo.owner, &repo.name)
            .await?
            .unwrap();
        assert_eq!(repo.default_strategy, MergeStrategy::Squash);

        Ok(())
    }

    #[tokio::test]
    async fn run_branch() -> Result<(), Box<dyn Error>> {
        let ctx = CoreContextTest::new();
        let repo = ctx
            .db_service
            .repositories_create(Repository {
                ..Default::default()
            })
            .await?;

        let base = RuleBranch::Named("base".into());

        AddMergeRule
            .run(
                &ctx.as_context(),
                &repo,
                base.clone(),
                RuleBranch::Wildcard,
                MergeStrategy::Squash,
            )
            .await?;

        let rule = ctx
            .db_service
            .merge_rules_get(&repo.owner, &repo.name, base.clone(), RuleBranch::Wildcard)
            .await?
            .unwrap();
        assert_eq!(rule.base_branch, base);
        assert_eq!(rule.head_branch, RuleBranch::Wildcard);

        Ok(())
    }
}
