use github_scbot_core::types::pulls::GhMergeStrategy;
use github_scbot_core::{config::Config, types::repository::RepositoryPath};
use github_scbot_macros::SCGetter;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, FromRow, Row};

use crate::fields::GhMergeStrategyDecode;

#[derive(
    SCGetter, Debug, Clone, derive_builder::Builder, Serialize, Deserialize, PartialEq, Eq,
)]
#[builder(default, setter(into))]
pub struct Repository {
    #[get]
    pub(crate) id: u64,
    #[get_deref]
    pub(crate) owner: String,
    #[get_deref]
    pub(crate) name: String,
    #[get]
    pub(crate) manual_interaction: bool,
    #[get_deref]
    pub(crate) pr_title_validation_regex: String,
    #[get]
    pub(crate) default_strategy: GhMergeStrategy,
    #[get]
    pub(crate) default_needed_reviewers_count: u64,
    #[get]
    pub(crate) default_automerge: bool,
    #[get]
    pub(crate) default_enable_qa: bool,
    #[get]
    pub(crate) default_enable_checks: bool,
}

impl Default for Repository {
    fn default() -> Self {
        Self {
            id: 0,
            owner: String::new(),
            name: String::new(),
            manual_interaction: false,
            pr_title_validation_regex: String::new(),
            default_strategy: GhMergeStrategy::Merge,
            default_needed_reviewers_count: 0,
            default_automerge: false,
            default_enable_qa: false,
            default_enable_checks: true,
        }
    }
}

impl Repository {
    pub fn builder() -> RepositoryBuilder {
        RepositoryBuilder::default()
    }

    pub fn path(&self) -> RepositoryPath {
        RepositoryPath::new_from_components(&self.owner, &self.name)
    }
}

impl RepositoryBuilder {
    pub fn with_config(&mut self, config: &Config) -> &mut Self {
        self.default_strategy = (&config.default_merge_strategy).try_into().ok();
        self.default_needed_reviewers_count = Some(config.default_needed_reviewers_count);
        self.pr_title_validation_regex = Some(config.default_pr_title_validation_regex.clone());
        self
    }
}

impl<'r> FromRow<'r, PgRow> for Repository {
    fn from_row(row: &'r PgRow) -> core::result::Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.try_get::<i32, _>("id")? as u64,
            owner: row.try_get("owner")?,
            name: row.try_get("name")?,
            manual_interaction: row.try_get("manual_interaction")?,
            pr_title_validation_regex: row.try_get("pr_title_validation_regex")?,
            default_strategy: *row.try_get::<GhMergeStrategyDecode, _>("default_strategy")?,
            default_needed_reviewers_count: row
                .try_get::<i32, _>("default_needed_reviewers_count")?
                as u64,
            default_automerge: row.try_get("default_automerge")?,
            default_enable_qa: row.try_get("default_enable_qa")?,
            default_enable_checks: row.try_get("default_enable_checks")?,
        })
    }
}

#[cfg(test)]
mod new_tests {
    use crate::{utils::db_test_case, Repository};

    #[actix_rt::test]
    async fn create() {
        db_test_case("repository_create", |mut db| async move {
            let repo = db
                .repositories_create(Repository::builder().owner("me").name("repo").build()?)
                .await?;

            assert_eq!(repo.owner(), "me");
            assert_eq!(repo.name(), "repo");

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn update() {
        db_test_case("repository_update", |mut db| async move {
            let repo = db
                .repositories_create(Repository::builder().owner("me").name("repo").build()?)
                .await?;

            let repo = db
                .repositories_update(
                    Repository::builder()
                        .id(repo.id())
                        .owner("me")
                        .name("repo2")
                        .build()?,
                )
                .await?;

            assert_eq!(repo.owner(), "me");
            assert_eq!(repo.name(), "repo2");

            Ok(())
        })
        .await;
    }

    // TODO: Write more tests
}
