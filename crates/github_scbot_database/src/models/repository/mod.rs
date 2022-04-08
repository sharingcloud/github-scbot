//! Repository model.

use std::convert::TryInto;

use github_scbot_conf::Config;
use github_scbot_database_macros::SCGetter;
use github_scbot_types::{common::GhRepository, pulls::GhMergeStrategy};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{
    errors::{DatabaseError, Result},
    schema::repository,
};

mod adapter;
mod builder;
pub use adapter::{DummyRepositoryDbAdapter, IRepositoryDbAdapter, RepositoryDbAdapter};
use builder::RepositoryModelBuilder;

/// Repository model.
#[derive(
    Debug, Deserialize, Serialize, Queryable, Identifiable, PartialEq, Clone, Eq, Default, SCGetter,
)]
#[table_name = "repository"]
pub struct RepositoryModel {
    /// Database ID.
    #[get]
    id: i32,
    /// Repository name.
    #[get_ref]
    name: String,
    /// Repository owner.
    #[get_ref]
    owner: String,
    default_strategy: String,
    /// Default reviewers count needed for a pull request.
    #[get]
    default_needed_reviewers_count: i32,
    /// Validation regex for pull request titles.
    #[get_ref]
    pr_title_validation_regex: String,
    /// Manual interaction.
    #[get]
    manual_interaction: bool,
    /// Default automerge.
    #[get]
    default_automerge: bool,
    /// Enable QA on this repository.
    #[get]
    default_enable_qa: bool,
    /// Enable checks on this repository.
    #[get]
    default_enable_checks: bool,
}

#[derive(Debug, Identifiable, Clone, AsChangeset, Default)]
#[table_name = "repository"]
pub struct RepositoryUpdate {
    /// Database ID.
    pub id: i32,
    /// Repository name.
    pub name: Option<String>,
    /// Repository owner.
    pub owner: Option<String>,
    /// Default merge strategy.
    pub default_strategy: Option<String>,
    /// Default needed reviewers count.
    pub default_needed_reviewers_count: Option<i32>,
    /// Title validation regex.
    pub pr_title_validation_regex: Option<String>,
    /// Manual interaction.
    pub manual_interaction: Option<bool>,
    /// Default automerge.
    pub default_automerge: Option<bool>,
    /// Default QA status.
    pub default_enable_qa: Option<bool>,
    /// Default check status.
    pub default_enable_checks: Option<bool>,
}

#[derive(Debug, Insertable)]
#[table_name = "repository"]
pub struct RepositoryCreation {
    pub name: String,
    pub owner: String,
    pub default_strategy: String,
    pub default_needed_reviewers_count: i32,
    pub pr_title_validation_regex: String,
    pub manual_interaction: bool,
    pub default_automerge: bool,
    pub default_enable_qa: bool,
    pub default_enable_checks: bool,
}

impl From<RepositoryCreation> for RepositoryModel {
    fn from(creation: RepositoryCreation) -> Self {
        Self {
            id: 0,
            name: creation.name,
            owner: creation.owner,
            default_strategy: creation.default_strategy,
            default_needed_reviewers_count: creation.default_needed_reviewers_count,
            pr_title_validation_regex: creation.pr_title_validation_regex,
            manual_interaction: creation.manual_interaction,
            default_automerge: creation.default_automerge,
            default_enable_qa: creation.default_enable_qa,
            default_enable_checks: creation.default_enable_checks,
        }
    }
}

impl RepositoryModel {
    /// Create builder.
    pub fn builder<'a>(config: &'a Config, owner: &str, name: &str) -> RepositoryModelBuilder<'a> {
        RepositoryModelBuilder::new(config, owner, name)
    }

    /// Prepare an update builder.
    pub fn create_update<'a>(&self) -> RepositoryModelBuilder<'a> {
        RepositoryModelBuilder::with_id(self.id)
    }

    /// Apply local update on repository.
    /// Result will not be in database.
    pub fn apply_local_update(&mut self, update: RepositoryUpdate) {
        if let Some(s) = update.name {
            self.name = s;
        }

        if let Some(s) = update.owner {
            self.owner = s;
        }

        if let Some(s) = update.default_strategy {
            self.default_strategy = s;
        }

        if let Some(s) = update.default_needed_reviewers_count {
            self.default_needed_reviewers_count = s;
        }

        if let Some(s) = update.pr_title_validation_regex {
            self.pr_title_validation_regex = s;
        }

        if let Some(s) = update.manual_interaction {
            self.manual_interaction = s;
        }

        if let Some(s) = update.default_automerge {
            self.default_automerge = s;
        }

        if let Some(s) = update.default_enable_qa {
            self.default_enable_qa = s;
        }

        if let Some(s) = update.default_enable_checks {
            self.default_enable_checks = s;
        }
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

    /// Create or update repository from GitHub object.
    #[tracing::instrument(skip(config, db_adapter))]
    pub async fn create_or_update_from_github(
        config: Config,
        db_adapter: &dyn IRepositoryDbAdapter,
        repository: &GhRepository,
    ) -> Result<Self> {
        let repository = repository.clone();

        RepositoryModel::builder_from_github(&config, &repository)
            .create_or_update(db_adapter)
            .await
            .map_err(DatabaseError::from)
    }

    /// Get repository from path.
    pub async fn get_from_path(db_adapter: &dyn IRepositoryDbAdapter, path: &str) -> Result<Self> {
        let path = path.to_owned();

        let (owner, name) = Self::extract_owner_and_name_from_path(&path)?;
        db_adapter.get_from_owner_and_name(owner, name).await
    }

    /// Get default merge strategy.
    pub fn default_merge_strategy(&self) -> GhMergeStrategy {
        if let Ok(strategy) = (&self.default_strategy[..]).try_into() {
            strategy
        } else {
            error!(
                repository_id = self.id,
                default_strategy = %self.default_strategy,
                message = "Invalid default merge strategy"
            );

            GhMergeStrategy::Merge
        }
    }

    /// Set default merge strategy.
    pub fn set_default_merge_strategy(&mut self, strategy: GhMergeStrategy) {
        self.default_strategy = strategy.to_string();
    }

    /// Get repository path.
    pub fn path(&self) -> String {
        format!("{}/{}", self.owner, self.name)
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
    use pretty_assertions::assert_eq;

    use crate::{
        models::{
            repository::{IRepositoryDbAdapter, RepositoryDbAdapter},
            RepositoryModel,
        },
        tests::using_test_db,
        DatabaseError, Result,
    };

    #[actix_rt::test]
    async fn create_repository() -> Result<()> {
        using_test_db("test_db_repository", |config, pool| async move {
            let db_adapter = RepositoryDbAdapter::new(pool.clone());
            let repo = RepositoryModel::builder(&config, "me", "TestRepo")
                .create_or_update(&db_adapter)
                .await?;

            assert_eq!(
                repo,
                RepositoryModel {
                    id: repo.id,
                    name: "TestRepo".into(),
                    owner: "me".into(),
                    default_strategy: config.default_merge_strategy.clone(),
                    default_needed_reviewers_count: config.default_needed_reviewers_count as i32,
                    pr_title_validation_regex: config.default_pr_title_validation_regex.clone(),
                    manual_interaction: false,
                    default_automerge: false,
                    default_enable_qa: true,
                    default_enable_checks: true,
                }
            );

            RepositoryModel::builder(&config, "me", "AnotherRepo")
                .create_or_update(&db_adapter)
                .await?;

            let repos = db_adapter.list().await?;
            assert_eq!(repos.len(), 2);

            Ok::<_, DatabaseError>(())
        })
        .await
    }
}
