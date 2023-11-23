use async_trait::async_trait;
use github_scbot_database_interface::DbService;
use github_scbot_domain_models::{MergeRule, MergeStrategy, Repository, RuleBranch};

use crate::Result;

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait(?Send)]
pub trait AddMergeRuleUseCaseInterface {
    async fn run(
        &self,
        repository: &Repository,
        base: RuleBranch,
        head: RuleBranch,
        strategy: MergeStrategy,
    ) -> Result<()>;
}

pub struct AddMergeRuleUseCase<'a> {
    pub db_service: &'a dyn DbService,
}

#[async_trait(?Send)]
impl<'a> AddMergeRuleUseCaseInterface for AddMergeRuleUseCase<'a> {
    async fn run(
        &self,
        repository: &Repository,
        base: RuleBranch,
        head: RuleBranch,
        strategy: MergeStrategy,
    ) -> Result<()> {
        let owner = &repository.owner;
        let name = &repository.name;

        if base == RuleBranch::Wildcard && head == RuleBranch::Wildcard {
            self.db_service
                .repositories_set_default_strategy(owner, name, strategy)
                .await?;
        } else {
            self.db_service
                .merge_rules_delete(owner, name, base.clone(), head.clone())
                .await?;
            self.db_service
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

    use github_scbot_database_interface::DbService;
    use github_scbot_database_memory::MemoryDb;
    use github_scbot_domain_models::{MergeStrategy, Repository, RuleBranch};

    use super::{AddMergeRuleUseCase, AddMergeRuleUseCaseInterface};

    #[tokio::test]
    async fn run_default() -> Result<(), Box<dyn Error>> {
        let db = MemoryDb::new();
        let repo = db
            .repositories_create(Repository {
                default_strategy: MergeStrategy::Merge,
                ..Default::default()
            })
            .await?;

        AddMergeRuleUseCase { db_service: &db }
            .run(
                &repo,
                RuleBranch::Wildcard,
                RuleBranch::Wildcard,
                MergeStrategy::Squash,
            )
            .await?;

        let repo = db.repositories_get(&repo.owner, &repo.name).await?.unwrap();
        assert_eq!(repo.default_strategy, MergeStrategy::Squash);

        Ok(())
    }

    #[tokio::test]
    async fn run_branch() -> Result<(), Box<dyn Error>> {
        let db = MemoryDb::new();
        let repo = db
            .repositories_create(Repository {
                ..Default::default()
            })
            .await?;

        let base = RuleBranch::Named("base".into());

        AddMergeRuleUseCase { db_service: &db }
            .run(
                &repo,
                base.clone(),
                RuleBranch::Wildcard,
                MergeStrategy::Squash,
            )
            .await?;

        let rule = db
            .merge_rules_get(&repo.owner, &repo.name, base.clone(), RuleBranch::Wildcard)
            .await?
            .unwrap();
        assert_eq!(rule.base_branch, base);
        assert_eq!(rule.head_branch, RuleBranch::Wildcard);

        Ok(())
    }
}
