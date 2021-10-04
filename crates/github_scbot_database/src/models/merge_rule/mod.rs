//! Merge rule model.

use std::convert::TryInto;

use github_scbot_database_macros::SCGetter;
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
    Debug, Deserialize, Serialize, Queryable, Identifiable, Clone, PartialEq, Eq, Default, SCGetter,
)]
#[table_name = "merge_rule"]
pub struct MergeRuleModel {
    /// Merge rule ID.
    #[get]
    id: i32,
    /// Repository ID.
    #[get]
    repository_id: i32,
    /// Base branch name.
    #[get_ref]
    base_branch: String,
    /// Head branch name.
    #[get_ref]
    head_branch: String,
    strategy: String,
}

#[derive(Debug, Identifiable, Clone, AsChangeset, Default)]
#[table_name = "merge_rule"]
pub struct MergeRuleUpdate {
    /// Database ID.
    pub id: i32,
    /// Base branch name.
    pub base_branch: Option<String>,
    /// Head branch name.
    pub head_branch: Option<String>,
    /// Strategy.
    pub strategy: Option<String>,
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
        MergeRuleBuilder::new(repo_model, base_branch.into(), head_branch.into())
    }

    /// Prepare an update builder.
    pub fn create_update<'a>(&self) -> MergeRuleBuilder<'a> {
        MergeRuleBuilder::with_id(self.id)
    }

    /// Apply local update on merge rule.
    /// Result will not be in database.
    pub fn apply_local_update(&mut self, update: MergeRuleUpdate) {
        if let Some(s) = update.base_branch {
            self.base_branch = s;
        }

        if let Some(s) = update.head_branch {
            self.head_branch = s;
        }

        if let Some(s) = update.strategy {
            self.strategy = s;
        }
    }

    /// Create builder from model.
    pub fn builder_from_model<'a>(
        repo_model: &'a RepositoryModel,
        model: &MergeRuleModel,
    ) -> MergeRuleBuilder<'a> {
        MergeRuleBuilder::from_model(repo_model, model)
    }

    /// Get strategy.
    pub fn strategy(&self) -> GhMergeStrategy {
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
        .map(|x| x.strategy())
        {
            v
        } else if let Ok(v) = Self::get_from_branches(
            db_adapter,
            repository,
            base_branch.clone(),
            RuleBranch::Wildcard,
        )
        .await
        .map(|x| x.strategy())
        {
            v
        } else {
            repository.default_merge_strategy()
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
                    repository_id: repo.id(),
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
                    repository_id: repo.id(),
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
