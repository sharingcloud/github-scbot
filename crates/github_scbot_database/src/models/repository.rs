//! Database repository models.

use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    errors::{DatabaseError, Result},
    schema::repository::{self, dsl},
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
}

impl Default for RepositoryModel {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            owner: String::new(),
            pr_title_validation_regex: String::new(),
            default_needed_reviewers_count: 2,
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
        diesel::insert_into(dsl::repository)
            .values(&entry)
            .execute(conn)?;

        Self::get_from_owner_and_name(conn, &entry.owner, &entry.name).ok_or_else(|| {
            DatabaseError::UnknownRepositoryError(format!("{}/{}", entry.owner, entry.name))
        })
    }

    /// List repositories.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    pub fn list(conn: &DbConn) -> Result<Vec<Self>> {
        dsl::repository.load::<Self>(conn).map_err(Into::into)
    }

    /// Get repository from database ID.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `id` - Repository ID
    pub fn get_from_id(conn: &DbConn, id: i32) -> Option<Self> {
        dsl::repository.filter(dsl::id.eq(id)).first(conn).ok()
    }

    /// Get repository from owner and name.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `owner` - Repository owner
    /// * `name` - Repository name
    pub fn get_from_owner_and_name(conn: &DbConn, owner: &str, name: &str) -> Option<Self> {
        dsl::repository
            .filter(dsl::name.eq(name))
            .filter(dsl::owner.eq(owner))
            .first(conn)
            .ok()
    }

    /// Get repository from path.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `path` - Repository path
    pub fn get_from_path(conn: &DbConn, path: &str) -> Result<Option<Self>> {
        let (owner, name) = Self::extract_owner_and_name_from_path(path)?;
        Ok(Self::get_from_owner_and_name(conn, owner, name))
    }

    /// Get or create a repository.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `entry` - Repository creation entry
    pub fn get_or_create(conn: &DbConn, entry: RepositoryCreation) -> Result<Self> {
        Self::get_from_owner_and_name(conn, &entry.owner, &entry.name)
            .map_or_else(|| Self::create(conn, entry), Ok)
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

        Err(DatabaseError::BadRepositoryPathError(path.to_string()))
    }
}
