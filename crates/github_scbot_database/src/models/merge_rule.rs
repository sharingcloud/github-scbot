//! Merge rule model.

use std::convert::TryInto;

use diesel::prelude::*;
use github_scbot_types::pulls::GHMergeStrategy;
use serde::{Deserialize, Serialize};

use crate::{
    errors::{DatabaseError, Result},
    schema::merge_rule,
    DbConn,
};

use super::RepositoryModel;

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
    Debug, Deserialize, Serialize, Queryable, Identifiable, Clone, AsChangeset, PartialEq, Eq,
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
struct MergeRuleCreation {
    pub repository_id: i32,
    pub base_branch: String,
    pub head_branch: String,
    pub strategy: String,
}

impl From<&MergeRuleModel> for MergeRuleCreation {
    fn from(model: &MergeRuleModel) -> Self {
        Self {
            repository_id: model.repository_id,
            base_branch: model.base_branch.clone(),
            head_branch: model.head_branch.clone(),
            strategy: model.strategy.clone(),
        }
    }
}

#[must_use]
pub struct MergeRuleBuilder<'a> {
    repo_model: &'a RepositoryModel,
    base_branch: RuleBranch,
    head_branch: RuleBranch,
    strategy: Option<GHMergeStrategy>,
}

impl<'a> MergeRuleBuilder<'a> {
    pub fn default<T1: Into<RuleBranch>, T2: Into<RuleBranch>>(
        repo_model: &'a RepositoryModel,
        base_branch: T1,
        head_branch: T2,
    ) -> Self {
        Self {
            repo_model,
            base_branch: base_branch.into(),
            head_branch: head_branch.into(),
            strategy: None,
        }
    }

    pub fn from_model(repo_model: &'a RepositoryModel, model: &MergeRuleModel) -> Self {
        Self {
            repo_model,
            base_branch: (&model.base_branch[..]).into(),
            head_branch: (&model.head_branch[..]).into(),
            strategy: Some(model.get_strategy()),
        }
    }

    pub fn strategy(mut self, strategy: GHMergeStrategy) -> Self {
        self.strategy = Some(strategy);
        self
    }

    pub fn build(&self) -> MergeRuleModel {
        MergeRuleModel {
            id: -1,
            repository_id: self.repo_model.id,
            base_branch: match self.base_branch.clone() {
                RuleBranch::Named(named) => named,
                RuleBranch::Wildcard => "*".into(),
            },
            head_branch: match self.head_branch.clone() {
                RuleBranch::Named(named) => named,
                RuleBranch::Wildcard => "*".into(),
            },
            strategy: self.strategy.unwrap_or(GHMergeStrategy::Merge).to_string(),
        }
    }

    pub fn create_or_update(self, conn: &DbConn) -> Result<MergeRuleModel> {
        let mut handle = match MergeRuleModel::get_from_branches(
            conn,
            self.repo_model,
            self.base_branch.clone(),
            self.head_branch.clone(),
        ) {
            Ok(entry) => entry,
            Err(_) => {
                let entry = self.build();
                MergeRuleModel::create(conn, (&entry).into())?
            }
        };

        handle.base_branch = self.base_branch.name();
        handle.head_branch = self.head_branch.name();
        handle.strategy = match self.strategy {
            Some(s) => s.to_string(),
            None => handle.strategy,
        };
        handle.save(conn)?;

        Ok(handle)
    }
}

impl MergeRuleModel {
    /// Create builder.
    ///
    /// # Arguments
    ///
    /// * `repo_model` - Repository
    /// * `base_branch` - Base branch
    /// * `head_branch` - Head branch
    pub fn builder<T1: Into<RuleBranch>, T2: Into<RuleBranch>>(
        repo_model: &RepositoryModel,
        base_branch: T1,
        head_branch: T2,
    ) -> MergeRuleBuilder {
        MergeRuleBuilder::default(repo_model, base_branch.into(), head_branch.into())
    }

    /// Create builder from model.
    ///
    /// # Arguments
    ///
    /// * `repo_model` - Repository
    /// * `model` - Rule model
    pub fn builder_from_model<'a>(
        repo_model: &'a RepositoryModel,
        model: &MergeRuleModel,
    ) -> MergeRuleBuilder<'a> {
        MergeRuleBuilder::from_model(repo_model, model)
    }

    fn create(conn: &DbConn, entry: MergeRuleCreation) -> Result<Self> {
        diesel::insert_into(merge_rule::table)
            .values(&entry)
            .get_result(conn)
            .map_err(Into::into)
    }

    /// Get strategy.
    pub fn get_strategy(&self) -> GHMergeStrategy {
        (&self.strategy[..]).try_into().unwrap()
    }

    /// Set strategy.
    ///
    /// # Arguments
    ///
    /// * `strategy` - Merge strategy
    pub fn set_strategy(&mut self, strategy: GHMergeStrategy) {
        self.strategy = strategy.to_string();
    }

    /// Get merge rule for branches.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `repository` - Repository
    /// * `base_branch` - Base branch
    /// * `head_branch` - Head branch
    pub fn get_from_branches<T1: Into<RuleBranch>, T2: Into<RuleBranch>>(
        conn: &DbConn,
        repository: &RepositoryModel,
        base_branch: T1,
        head_branch: T2,
    ) -> Result<Self> {
        let base_branch = base_branch.into();
        let head_branch = head_branch.into();

        merge_rule::table
            .filter(merge_rule::repository_id.eq(repository.id))
            .filter(merge_rule::base_branch.eq(base_branch.name()))
            .filter(merge_rule::head_branch.eq(head_branch.name()))
            .first(conn)
            .map_err(|_e| {
                DatabaseError::UnknownMergeRule(
                    repository.get_path(),
                    base_branch.name(),
                    head_branch.name(),
                )
            })
    }

    /// List rules from repository ID.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `repository_id` - Repository ID
    pub fn list_from_repository_id(
        conn: &DbConn,
        repository_id: i32,
    ) -> Result<Vec<MergeRuleModel>> {
        let rules = merge_rule::table
            .filter(merge_rule::repository_id.eq(repository_id))
            .get_results(conn)?;

        Ok(rules)
    }

    /// List merge rules.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    pub fn list(conn: &DbConn) -> Result<Vec<Self>> {
        merge_rule::table.load::<Self>(conn).map_err(Into::into)
    }

    /// Remove merge rule.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    pub fn remove(&self, conn: &DbConn) -> Result<()> {
        diesel::delete(merge_rule::table.filter(merge_rule::id.eq(self.id))).execute(conn)?;

        Ok(())
    }

    /// Save model instance to database.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    pub fn save(&mut self, conn: &DbConn) -> Result<()> {
        self.save_changes::<Self>(conn)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_conf::Config;
    use pretty_assertions::assert_eq;

    use crate::establish_single_test_connection;

    use super::*;

    fn test_init() -> (Config, DbConn) {
        let config = Config::from_env();
        let conn = establish_single_test_connection(&config).unwrap();

        (config, conn)
    }

    #[test]
    fn create_and_update() {
        let (config, conn) = test_init();

        let repo = RepositoryModel::builder(&config, "me", "TestRepo")
            .create_or_update(&conn)
            .unwrap();

        let rule = MergeRuleModel::builder(&repo, "test", RuleBranch::Wildcard)
            .create_or_update(&conn)
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
            .strategy(GHMergeStrategy::Squash)
            .create_or_update(&conn)
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

        assert_eq!(MergeRuleModel::list(&conn).unwrap().len(), 1);
    }
}
