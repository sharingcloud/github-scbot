//! Database repository models.

use std::convert::TryInto;

use diesel::prelude::*;
use github_scbot_types::pulls::GHMergeStrategy;
use serde::{Deserialize, Serialize};

use crate::{
    errors::{DatabaseError, Result},
    schema::{merge_rule, repository},
    DbConn,
};

/// Repository model.
#[derive(
    Debug, Deserialize, Serialize, Queryable, Identifiable, AsChangeset, PartialEq, Clone, Eq,
)]
#[table_name = "repository"]
pub struct RepositoryModel {
    /// Database ID.
    pub id: i32,
    /// Repository name.
    pub name: String,
    /// Repository owner.
    pub owner: String,
    /// Validation regex for pull request titles.
    pub pr_title_validation_regex: String,
    /// Default reviewers count needed for a pull request.
    pub default_needed_reviewers_count: i32,
    /// Default strategy.
    default_strategy: String,
}

/// Repository creation.
#[derive(Debug, Insertable)]
#[table_name = "repository"]
pub struct RepositoryCreation {
    /// Repository name.
    pub name: String,
    /// Repository owner.
    pub owner: String,
    /// Validation regex for pull request titles.
    pub pr_title_validation_regex: String,
    /// Default reviewers count needed for a pull request.
    pub default_needed_reviewers_count: i32,
    /// Default strategy.
    pub default_strategy: String,
}

impl Default for RepositoryModel {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            owner: String::new(),
            pr_title_validation_regex: String::new(),
            default_needed_reviewers_count: 2,
            default_strategy: GHMergeStrategy::Merge.to_string(),
        }
    }
}

impl Default for RepositoryCreation {
    fn default() -> Self {
        Self {
            name: String::new(),
            owner: String::new(),
            pr_title_validation_regex: String::new(),
            default_needed_reviewers_count: 2,
            default_strategy: GHMergeStrategy::Merge.to_string(),
        }
    }
}

impl RepositoryModel {
    /// Create a repository.
    ///
    /// # Arguments
    ///
    /// * `entry` - Database entry
    pub fn create(conn: &DbConn, entry: RepositoryCreation) -> Result<Self> {
        diesel::insert_into(repository::table)
            .values(&entry)
            .execute(conn)?;

        Self::get_from_owner_and_name(conn, &entry.owner, &entry.name)
    }

    /// List repositories.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    pub fn list(conn: &DbConn) -> Result<Vec<Self>> {
        repository::table.load::<Self>(conn).map_err(Into::into)
    }

    /// Get repository from database ID.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `id` - Repository ID
    pub fn get_from_id(conn: &DbConn, id: i32) -> Result<Self> {
        repository::table
            .filter(repository::id.eq(id))
            .first(conn)
            .map_err(|_e| DatabaseError::UnknownRepository(format!("<ID {}>", id)))
    }

    /// Get repository from owner and name.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `owner` - Repository owner
    /// * `name` - Repository name
    pub fn get_from_owner_and_name(conn: &DbConn, owner: &str, name: &str) -> Result<Self> {
        repository::table
            .filter(repository::name.eq(name))
            .filter(repository::owner.eq(owner))
            .first(conn)
            .map_err(|_e| DatabaseError::UnknownRepository(format!("{0}/{1}", owner, name)))
    }

    /// Get repository from path.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `path` - Repository path
    pub fn get_from_path(conn: &DbConn, path: &str) -> Result<Self> {
        let (owner, name) = Self::extract_owner_and_name_from_path(path)?;
        Self::get_from_owner_and_name(conn, owner, name)
    }

    /// Get or create a repository.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `entry` - Repository creation entry
    pub fn get_or_create(conn: &DbConn, entry: RepositoryCreation) -> Result<Self> {
        match Self::get_from_owner_and_name(conn, &entry.owner, &entry.name) {
            Err(_) => Self::create(conn, entry),
            Ok(v) => Ok(v),
        }
    }

    /// Get default merge strategy.
    pub fn get_default_merge_strategy(&self) -> GHMergeStrategy {
        (&self.default_strategy[..]).try_into().unwrap()
    }

    /// Set default merge strategy.
    ///
    /// # Arguments
    ///
    /// * `strategy` - Merge strategy
    pub fn set_default_merge_strategy(&mut self, strategy: GHMergeStrategy) {
        self.default_strategy = strategy.to_string();
    }

    /// Get repository path.
    pub fn get_path(&self) -> String {
        format!("{}/{}", self.owner, self.name)
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

    /// Extract repository owner and name from path.
    ///
    /// # Arguments
    ///
    /// * `path` - Repository path
    pub fn extract_owner_and_name_from_path(path: &str) -> Result<(&str, &str)> {
        let mut split = path.split_terminator('/');
        let owner = split.next();
        let name = split.next();

        if let Some(owner) = owner {
            if let Some(name) = name {
                return Ok((owner, name));
            }
        }

        Err(DatabaseError::BadRepositoryPath(path.to_string()))
    }
}

/// Merge rule model.
#[derive(Debug, Deserialize, Serialize, Queryable, Identifiable, Clone, AsChangeset)]
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

/// Merge rule creation.
#[derive(Debug, Insertable)]
#[table_name = "merge_rule"]
pub struct MergeRuleCreation {
    /// Repository ID.
    pub repository_id: i32,
    /// Base branch name.
    pub base_branch: String,
    /// Head branch name.
    pub head_branch: String,
    /// Strategy name.
    pub strategy: String,
}

impl MergeRuleModel {
    /// Create merge rule from creation entry.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `entry` - Merge rule creation entry
    pub fn create(conn: &DbConn, entry: MergeRuleCreation) -> Result<Self> {
        diesel::insert_into(merge_rule::table)
            .values(&entry)
            .execute(conn)?;

        Self::get_from_branches(
            conn,
            entry.repository_id,
            &entry.base_branch,
            &entry.head_branch,
        )
    }

    /// Get or create a merge rule.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `entry` - Merge rule creation entry
    pub fn get_or_create(conn: &DbConn, entry: MergeRuleCreation) -> Result<Self> {
        match Self::get_from_branches(
            conn,
            entry.repository_id,
            &entry.base_branch,
            &entry.head_branch,
        ) {
            Ok(v) => Ok(v),
            Err(_) => Self::create(conn, entry),
        }
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
    /// * `repository_id` - Repository ID
    /// * `base_branch` - Base branch
    /// * `head_branch` - Head branch
    pub fn get_from_branches(
        conn: &DbConn,
        repository_id: i32,
        base_branch: &str,
        head_branch: &str,
    ) -> Result<Self> {
        merge_rule::table
            .filter(merge_rule::repository_id.eq(repository_id))
            .filter(merge_rule::base_branch.eq(base_branch))
            .filter(merge_rule::head_branch.eq(head_branch))
            .first(conn)
            .map_err(|_e| {
                DatabaseError::UnknownMergeRule(
                    format!("<ID {}>", repository_id),
                    base_branch.to_string(),
                    head_branch.to_string(),
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
