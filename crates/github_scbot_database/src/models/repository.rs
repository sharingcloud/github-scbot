//! Repository model.

use std::convert::{TryFrom, TryInto};

use diesel::prelude::*;
use github_scbot_conf::Config;
use github_scbot_types::{common::GhRepository, pulls::GhMergeStrategy};
use serde::{Deserialize, Serialize};

use crate::{
    errors::{DatabaseError, Result},
    schema::repository,
    DbConn, DbPool,
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
    /// Default strategy.
    default_strategy: String,
    /// Default reviewers count needed for a pull request.
    pub default_needed_reviewers_count: i32,
    /// Validation regex for pull request titles.
    pub pr_title_validation_regex: String,
    /// Manual interaction.
    pub manual_interaction: bool,
}

#[derive(Debug, Insertable)]
#[table_name = "repository"]
struct RepositoryCreation {
    pub name: String,
    pub owner: String,
    pub default_strategy: String,
    pub default_needed_reviewers_count: i32,
    pub pr_title_validation_regex: String,
    pub manual_interaction: bool,
}

#[must_use]
pub struct RepositoryModelBuilder<'a> {
    owner: String,
    name: String,
    config: &'a Config,
    default_strategy: Option<GhMergeStrategy>,
    default_needed_reviewers_count: Option<u64>,
    pr_title_validation_regex: Option<String>,
    manual_interaction: Option<bool>,
}

impl<'a> RepositoryModelBuilder<'a> {
    pub fn default(config: &'a Config, owner: &str, repo_name: &str) -> Self {
        Self {
            owner: owner.into(),
            name: repo_name.into(),
            config,
            default_strategy: None,
            default_needed_reviewers_count: None,
            pr_title_validation_regex: None,
            manual_interaction: None,
        }
    }

    pub fn from_model(config: &'a Config, model: &RepositoryModel) -> Self {
        Self {
            owner: model.owner.clone(),
            name: model.name.clone(),
            config,
            default_strategy: Some(model.get_default_merge_strategy()),
            default_needed_reviewers_count: Some(model.default_needed_reviewers_count as u64),
            pr_title_validation_regex: Some(model.pr_title_validation_regex.clone()),
            manual_interaction: Some(model.manual_interaction),
        }
    }

    pub fn from_github(config: &'a Config, repo: &GhRepository) -> Self {
        Self {
            owner: repo.owner.login.clone(),
            name: repo.name.clone(),
            config,
            default_strategy: None,
            default_needed_reviewers_count: None,
            pr_title_validation_regex: None,
            manual_interaction: None,
        }
    }

    pub fn pr_title_validation_regex<T: Into<String>>(mut self, regex: T) -> Self {
        self.pr_title_validation_regex = Some(regex.into());
        self
    }

    pub fn default_needed_reviewers_count(mut self, count: u64) -> Self {
        self.default_needed_reviewers_count = Some(count);
        self
    }

    pub fn default_strategy(mut self, strategy: GhMergeStrategy) -> Self {
        self.default_strategy = Some(strategy);
        self
    }

    pub fn manual_interaction(mut self, mode: bool) -> Self {
        self.manual_interaction = Some(mode);
        self
    }

    fn build(&self) -> RepositoryCreation {
        RepositoryCreation {
            owner: self.owner.clone(),
            name: self.name.clone(),
            pr_title_validation_regex: self
                .pr_title_validation_regex
                .clone()
                .unwrap_or_else(|| self.config.default_pr_title_validation_regex.clone()),
            default_needed_reviewers_count: self
                .default_needed_reviewers_count
                .unwrap_or(self.config.default_needed_reviewers_count)
                as i32,
            default_strategy: self
                .default_strategy
                .unwrap_or_else(|| {
                    GhMergeStrategy::try_from(&self.config.default_merge_strategy[..]).unwrap()
                })
                .to_string(),
            manual_interaction: self.manual_interaction.unwrap_or(false),
        }
    }

    pub fn create_or_update(self, conn: &DbConn) -> Result<RepositoryModel> {
        conn.transaction(|| {
            let mut handle =
                match RepositoryModel::get_from_owner_and_name(conn, &self.owner, &self.name) {
                    Ok(entry) => entry,
                    Err(_) => {
                        let entry = self.build();
                        RepositoryModel::create(conn, entry)?
                    }
                };

            handle.pr_title_validation_regex = match self.pr_title_validation_regex {
                Some(p) => p,
                None => handle.pr_title_validation_regex,
            };
            handle.default_needed_reviewers_count = match self.default_needed_reviewers_count {
                Some(d) => d as i32,
                None => handle.default_needed_reviewers_count,
            };
            handle.default_strategy = match self.default_strategy {
                Some(d) => d.to_string(),
                None => handle.default_strategy,
            };
            handle.manual_interaction = match self.manual_interaction {
                Some(m) => m,
                None => handle.manual_interaction,
            };
            handle.save(conn)?;

            Ok(handle)
        })
    }
}

impl RepositoryModel {
    /// Create builder.
    pub fn builder<'a>(config: &'a Config, owner: &str, name: &str) -> RepositoryModelBuilder<'a> {
        RepositoryModelBuilder::default(config, owner, name)
    }

    /// Create builder from model.
    pub fn builder_from_model<'a>(config: &'a Config, model: &Self) -> RepositoryModelBuilder<'a> {
        RepositoryModelBuilder::from_model(config, model)
    }

    /// Create builder from GitHub repository.
    pub fn builder_from_github<'a>(
        config: &'a Config,
        repo: &GhRepository,
    ) -> RepositoryModelBuilder<'a> {
        RepositoryModelBuilder::from_github(config, repo)
    }

    fn create(conn: &DbConn, entry: RepositoryCreation) -> Result<Self> {
        diesel::insert_into(repository::table)
            .values(&entry)
            .get_result(conn)
            .map_err(Into::into)
    }

    /// Create or update repository from GitHub object.
    pub async fn create_or_update_from_github(
        config: Config,
        pool: DbPool,
        repository: GhRepository,
    ) -> Result<Self> {
        actix_threadpool::run(move || {
            let conn = pool.get()?;
            RepositoryModel::builder_from_github(&config, &repository)
                .create_or_update(&conn)
                .map_err(DatabaseError::from)
        })
        .await
        .map_err(Into::into)
    }

    /// List repositories.
    pub fn list(conn: &DbConn) -> Result<Vec<Self>> {
        repository::table.load::<Self>(conn).map_err(Into::into)
    }

    /// Get repository from database ID.
    pub fn get_from_id(conn: &DbConn, id: i32) -> Result<Self> {
        repository::table
            .filter(repository::id.eq(id))
            .first(conn)
            .map_err(|_e| DatabaseError::UnknownRepository(format!("<ID {}>", id)))
    }

    /// Get repository from owner and name.
    pub fn get_from_owner_and_name(conn: &DbConn, owner: &str, name: &str) -> Result<Self> {
        repository::table
            .filter(repository::name.eq(name))
            .filter(repository::owner.eq(owner))
            .first(conn)
            .map_err(|_e| DatabaseError::UnknownRepository(format!("{0}/{1}", owner, name)))
    }

    /// Get repository from path.
    pub async fn get_from_path(pool: DbPool, path: String) -> Result<Self> {
        actix_threadpool::run(move || {
            let conn = pool.get()?;
            let (owner, name) = Self::extract_owner_and_name_from_path(&path)?;
            Self::get_from_owner_and_name(&conn, owner, name)
        })
        .await
        .map_err(Into::into)
    }

    /// Get default merge strategy.
    pub fn get_default_merge_strategy(&self) -> GhMergeStrategy {
        (&self.default_strategy[..]).try_into().unwrap()
    }

    /// Set default merge strategy.
    pub fn set_default_merge_strategy(&mut self, strategy: GhMergeStrategy) {
        self.default_strategy = strategy.to_string();
    }

    /// Get repository path.
    pub fn get_path(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }

    /// Save model instance to database.
    pub fn save(&mut self, conn: &DbConn) -> Result<()> {
        self.save_changes::<Self>(conn)?;

        Ok(())
    }

    /// Extract repository owner and name from path.
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

#[cfg(test)]
mod tests {
    use github_scbot_conf::Config;
    use pretty_assertions::assert_eq;

    use crate::{models::RepositoryModel, tests::using_test_db, DatabaseError, Result};

    #[actix_rt::test]
    async fn create_repository() -> Result<()> {
        let config = Config::from_env();

        using_test_db(&config.clone(), "test_db_repository", |pool| async move {
            let conn = pool.get()?;
            let repo =
                RepositoryModel::builder(&config, "me", "TestRepo").create_or_update(&conn)?;

            assert_eq!(
                repo,
                RepositoryModel {
                    id: repo.id,
                    name: "TestRepo".into(),
                    owner: "me".into(),
                    default_strategy: config.default_merge_strategy.clone(),
                    default_needed_reviewers_count: config.default_needed_reviewers_count as i32,
                    pr_title_validation_regex: config.default_pr_title_validation_regex.clone(),
                    manual_interaction: false
                }
            );

            RepositoryModel::builder(&config, "me", "AnotherRepo").create_or_update(&conn)?;

            let repos = RepositoryModel::list(&conn)?;
            assert_eq!(repos.len(), 2);

            Ok::<_, DatabaseError>(())
        })
        .await
    }
}
