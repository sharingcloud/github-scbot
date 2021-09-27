//! Repository model.

use std::convert::TryInto;

use github_scbot_conf::Config;
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
    Debug,
    Deserialize,
    Serialize,
    Queryable,
    Identifiable,
    AsChangeset,
    PartialEq,
    Clone,
    Eq,
    Default,
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
    /// Default automerge.
    pub default_automerge: bool,
    /// Enable QA on this repository.
    pub default_enable_qa: bool,
    /// Enable checks on this repository.
    pub default_enable_checks: bool,
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

    /// Create or update repository from GitHub object.
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
    pub fn get_default_merge_strategy(&self) -> GhMergeStrategy {
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
    pub fn get_path(&self) -> String {
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
