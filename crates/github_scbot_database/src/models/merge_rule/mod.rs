//! Merge rule model.

use std::convert::TryInto;

use github_scbot_types::pulls::GhMergeStrategy;
use serde::{Deserialize, Serialize};
use tracing::error;

use super::RepositoryModel;
use crate::{errors::Result, schema::merge_rule};

mod adapter;
mod builder;
pub use adapter::{DummyMergeRuleDbAdapter, IMergeRuleDbAdapter, MergeRuleDbAdapter};
use builder::MergeRuleBuilder;

/// Rule branch.
#[derive(Clone)]
pub enum RuleBranch {
    /// Named.
    Named(String),
    /// Wildcard.
    Wildcard,
}

impl From<&str> for RuleBranch {
    fn from(value: &str) -> Self {
        match value {
            "*" => Self::Wildcard,
            n => Self::Named(n.into()),
        }
    }
}

impl RuleBranch {
    /// Get branch name.
    pub fn name(&self) -> String {
        match self {
            RuleBranch::Named(s) => s.clone(),
            RuleBranch::Wildcard => "*".into(),
        }
    }
}

/// Merge rule model.
#[derive(
    Debug,
    Deserialize,
    Serialize,
    Queryable,
    Identifiable,
    Clone,
    AsChangeset,
    PartialEq,
    Eq,
    Default,
)]
#[table_name = "merge_rule"]
pub struct MergeRuleModel {
    /// Merge rule ID.
    pub id: i32,
    /// Repository ID.
    pub repository_id: i32,
    /// Base branch name.
    pub base_branch: String,
    /// Head branch name.
    pub head_branch: String,
    /// Strategy name.
    strategy: String,
}

#[derive(Debug, Insertable)]
#[table_name = "merge_rule"]
pub struct MergeRuleCreation {
    pub repository_id: i32,
    pub base_branch: String,
    pub head_branch: String,
    pub strategy: String,
}

impl From<MergeRuleModel> for MergeRuleCreation {
    fn from(model: MergeRuleModel) -> Self {
        Self {
            repository_id: model.repository_id,
            base_branch: model.base_branch,
            head_branch: model.head_branch,
            strategy: model.strategy,
        }
    }
}

impl From<MergeRuleCreation> for MergeRuleModel {
    fn from(creation: MergeRuleCreation) -> Self {
        Self {
            id: 0,
            repository_id: creation.repository_id,
            base_branch: creation.base_branch,
            head_branch: creation.head_branch,
            strategy: creation.strategy,
        }
    }
}

impl MergeRuleModel {
    /// Create builder.
    pub fn builder<T1: Into<RuleBranch>, T2: Into<RuleBranch>>(
        repo_model: &RepositoryModel,
        base_branch: T1,
        head_branch: T2,
    ) -> MergeRuleBuilder {
        MergeRuleBuilder::default(repo_model, base_branch.into(), head_branch.into())
    }

    /// Create builder from model.
    pub fn builder_from_model<'a>(
        repo_model: &'a RepositoryModel,
        model: &MergeRuleModel,
    ) -> MergeRuleBuilder<'a> {
        MergeRuleBuilder::from_model(repo_model, model)
    }

    /// Get strategy.
    pub fn get_strategy(&self) -> GhMergeStrategy {
        if let Ok(strategy) = (&self.strategy[..]).try_into() {
            strategy
        } else {
            error!(
                merge_rule_id = self.id,
                strategy = %self.strategy,
                message = "Invalid strategy"
            );

            GhMergeStrategy::Merge
        }
    }

    /// Set strategy.
    pub fn set_strategy(&mut self, strategy: GhMergeStrategy) {
        self.strategy = strategy.to_string();
    }

    /// Get merge rule for branches.
    pub async fn get_from_branches<T1: Into<RuleBranch>, T2: Into<RuleBranch>>(
        db_adapter: &dyn IMergeRuleDbAdapter,
        repository: &RepositoryModel,
        base_branch: T1,
        head_branch: T2,
    ) -> Result<Self> {
        db_adapter
            .get_from_branches(repository, &base_branch.into(), &head_branch.into())
            .await
    }

    /// Get merge rule for branches or get default.
    pub async fn get_strategy_from_branches<
        T1: Into<RuleBranch> + Clone,
        T2: Into<RuleBranch> + Clone,
    >(
        db_adapter: &dyn IMergeRuleDbAdapter,
        repository: &RepositoryModel,
        base_branch: T1,
        head_branch: T2,
    ) -> GhMergeStrategy {
        if let Ok(v) = Self::get_from_branches(
            db_adapter,
            repository,
            base_branch.clone(),
            head_branch.clone(),
        )
        .await
        .map(|x| x.get_strategy())
        {
            v
        } else if let Ok(v) = Self::get_from_branches(
            db_adapter,
            repository,
            base_branch.clone(),
            RuleBranch::Wildcard,
        )
        .await
        .map(|x| x.get_strategy())
        {
            v
        } else {
            repository.get_default_merge_strategy()
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{models::repository::RepositoryDbAdapter, tests::using_test_db, DatabaseError};

    #[actix_rt::test]
    async fn create_and_update() -> Result<()> {
        using_test_db("test_db_merge_rule", |config, pool| async move {
            let repo_db_adapter = RepositoryDbAdapter::new(pool.clone());
            let db_adapter = MergeRuleDbAdapter::new(pool.clone());

            let repo = RepositoryModel::builder(&config, "me", "TestRepo")
                .create_or_update(&repo_db_adapter)
                .await
                .unwrap();

            let rule = MergeRuleModel::builder(&repo, "test", RuleBranch::Wildcard)
                .create_or_update(&db_adapter)
                .await
                .unwrap();

            assert_eq!(
                rule,
                MergeRuleModel {
                    id: rule.id,
                    repository_id: repo.id,
                    base_branch: "test".into(),
                    head_branch: "*".into(),
                    strategy: "merge".into()
                }
            );

            let rule = MergeRuleModel::builder(&repo, "test", RuleBranch::Wildcard)
                .strategy(GhMergeStrategy::Squash)
                .create_or_update(&db_adapter)
                .await
                .unwrap();

            assert_eq!(
                rule,
                MergeRuleModel {
                    id: rule.id,
                    repository_id: repo.id,
                    base_branch: "test".into(),
                    head_branch: "*".into(),
                    strategy: "squash".into()
                }
            );

            assert_eq!(db_adapter.list().await.unwrap().len(), 1);
            Ok::<_, DatabaseError>(())
        })
        .await
    }
}
