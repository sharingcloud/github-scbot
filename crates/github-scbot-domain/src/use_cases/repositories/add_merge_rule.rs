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
