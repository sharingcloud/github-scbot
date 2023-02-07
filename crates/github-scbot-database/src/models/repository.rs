use github_scbot_core::types::pulls::GhMergeStrategy;
use github_scbot_core::{config::Config, types::repository::RepositoryPath};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, FromRow, Row};

use crate::fields::GhMergeStrategyDecode;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Repository {
    pub id: u64,
    pub owner: String,
    pub name: String,
    pub manual_interaction: bool,
    pub pr_title_validation_regex: String,
    pub default_strategy: GhMergeStrategy,
    pub default_needed_reviewers_count: u64,
    pub default_automerge: bool,
    pub default_enable_qa: bool,
    pub default_enable_checks: bool,
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
    pub fn path(&self) -> RepositoryPath {
        RepositoryPath::new_from_components(&self.owner, &self.name)
    }

    pub fn with_config(mut self, config: &Config) -> Self {
        self.default_strategy = (&config.default_merge_strategy)
            .try_into()
            .unwrap_or_default();
        self.default_needed_reviewers_count = config.default_needed_reviewers_count;
        self.pr_title_validation_regex = config.default_pr_title_validation_regex.clone();
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
    use github_scbot_core::types::pulls::GhMergeStrategy;

    use crate::{utils::db_test_case, DatabaseError, Repository};

    #[actix_rt::test]
    async fn create() {
        db_test_case("repository_create", |mut db| async move {
            let repo = db
                .repositories_create(Repository {
                    owner: "me".into(),
                    name: "repo".into(),
                    ..Default::default()
                })
                .await?;

            assert_eq!(repo.owner, "me");
            assert_eq!(repo.name, "repo");

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn update() {
        db_test_case("repository_update", |mut db| async move {
            let repo = db
                .repositories_create(Repository {
                    owner: "me".into(),
                    name: "repo".into(),
                    ..Default::default()
                })
                .await?;

            let repo = db
                .repositories_update(Repository {
                    id: repo.id,
                    owner: "me".into(),
                    name: "repo2".into(),
                    ..Default::default()
                })
                .await?;

            assert_eq!(repo.owner, "me");
            assert_eq!(repo.name, "repo2");

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn get() {
        db_test_case("repository_get", |mut db| async move {
            assert_eq!(db.repositories_get("me", "repo").await?, None);

            let repo = db
                .repositories_create(Repository {
                    owner: "me".into(),
                    name: "repo".into(),
                    ..Default::default()
                })
                .await?;

            let get_repo = db.repositories_get("me", "repo").await?;
            assert_eq!(get_repo, Some(repo));

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn get_from_id() {
        db_test_case("repository_get_from_id", |mut db| async move {
            assert_eq!(db.repositories_get_from_id(1).await?, None);

            let repo = db
                .repositories_create(Repository {
                    owner: "me".into(),
                    name: "repo".into(),
                    ..Default::default()
                })
                .await?;

            let get_repo = db.repositories_get_from_id(repo.id).await?;
            assert_eq!(get_repo, Some(repo));

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn delete() {
        db_test_case("repository_delete", |mut db| async move {
            assert!(!db.repositories_delete("me", "repo").await?);

            db.repositories_create(Repository {
                owner: "me".into(),
                name: "repo".into(),
                ..Default::default()
            })
            .await?;

            assert!(db.repositories_delete("me", "repo").await?);
            assert_eq!(db.repositories_get("me", "repo").await?, None);

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn set_manual_interaction() {
        db_test_case("repository_set_manual_interaction", |mut db| async move {
            assert!(matches!(
                db.repositories_set_manual_interaction("me", "repo", true)
                    .await,
                Err(DatabaseError::UnknownRepository(_))
            ));

            db.repositories_create(Repository {
                owner: "me".into(),
                name: "repo".into(),
                manual_interaction: false,
                ..Default::default()
            })
            .await?;

            let repo = db
                .repositories_set_manual_interaction("me", "repo", true)
                .await?;
            assert!(repo.manual_interaction);

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn set_pr_title_validation_regex() {
        db_test_case(
            "repository_set_pr_title_validation_regex",
            |mut db| async move {
                assert!(matches!(
                    db.repositories_set_pr_title_validation_regex("me", "repo", "[a-z]+")
                        .await,
                    Err(DatabaseError::UnknownRepository(_))
                ));

                db.repositories_create(Repository {
                    owner: "me".into(),
                    name: "repo".into(),
                    pr_title_validation_regex: "".into(),
                    ..Default::default()
                })
                .await?;

                let repo = db
                    .repositories_set_pr_title_validation_regex("me", "repo", "[a-z]+")
                    .await?;
                assert_eq!(repo.pr_title_validation_regex, "[a-z]+");

                Ok(())
            },
        )
        .await;
    }

    #[actix_rt::test]
    async fn set_default_strategy() {
        db_test_case("repository_set_default_strategy", |mut db| async move {
            assert!(matches!(
                db.repositories_set_default_strategy("me", "repo", GhMergeStrategy::Squash)
                    .await,
                Err(DatabaseError::UnknownRepository(_))
            ));

            db.repositories_create(Repository {
                owner: "me".into(),
                name: "repo".into(),
                default_strategy: GhMergeStrategy::Merge,
                ..Default::default()
            })
            .await?;

            let repo = db
                .repositories_set_default_strategy("me", "repo", GhMergeStrategy::Squash)
                .await?;
            assert_eq!(repo.default_strategy, GhMergeStrategy::Squash);

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn set_default_needed_reviewers_count() {
        db_test_case(
            "repository_set_default_needed_reviewers_count",
            |mut db| async move {
                assert!(matches!(
                    db.repositories_set_default_needed_reviewers_count("me", "repo", 1)
                        .await,
                    Err(DatabaseError::UnknownRepository(_))
                ));

                db.repositories_create(Repository {
                    owner: "me".into(),
                    name: "repo".into(),
                    default_needed_reviewers_count: 0,
                    ..Default::default()
                })
                .await?;

                let repo = db
                    .repositories_set_default_needed_reviewers_count("me", "repo", 1)
                    .await?;
                assert_eq!(repo.default_needed_reviewers_count, 1);

                Ok(())
            },
        )
        .await;
    }

    #[actix_rt::test]
    async fn set_default_automerge() {
        db_test_case("repository_set_default_automerge", |mut db| async move {
            assert!(matches!(
                db.repositories_set_default_automerge("me", "repo", true)
                    .await,
                Err(DatabaseError::UnknownRepository(_))
            ));

            db.repositories_create(Repository {
                owner: "me".into(),
                name: "repo".into(),
                default_automerge: false,
                ..Default::default()
            })
            .await?;

            let repo = db
                .repositories_set_default_automerge("me", "repo", true)
                .await?;
            assert!(repo.default_automerge);

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn set_default_enable_qa() {
        db_test_case("repository_set_default_enable_qa", |mut db| async move {
            assert!(matches!(
                db.repositories_set_default_enable_qa("me", "repo", true)
                    .await,
                Err(DatabaseError::UnknownRepository(_))
            ));

            db.repositories_create(Repository {
                owner: "me".into(),
                name: "repo".into(),
                default_enable_qa: false,
                ..Default::default()
            })
            .await?;

            let repo = db
                .repositories_set_default_enable_qa("me", "repo", true)
                .await?;
            assert!(repo.default_enable_qa);

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn set_default_enable_checks() {
        db_test_case(
            "repository_set_default_enable_checks",
            |mut db| async move {
                assert!(matches!(
                    db.repositories_set_default_enable_checks("me", "repo", true)
                        .await,
                    Err(DatabaseError::UnknownRepository(_))
                ));

                db.repositories_create(Repository {
                    owner: "me".into(),
                    name: "repo".into(),
                    default_enable_checks: false,
                    ..Default::default()
                })
                .await?;

                let repo = db
                    .repositories_set_default_enable_checks("me", "repo", true)
                    .await?;
                assert!(repo.default_enable_checks);

                Ok(())
            },
        )
        .await;
    }

    #[actix_rt::test]
    async fn all() {
        db_test_case("repository_all", |mut db| async move {
            assert_eq!(db.repositories_all().await?, vec![]);

            let repo1 = db
                .repositories_create(Repository {
                    owner: "me".into(),
                    name: "repo".into(),
                    ..Default::default()
                })
                .await?;

            let repo2 = db
                .repositories_create(Repository {
                    owner: "me".into(),
                    name: "repo2".into(),
                    ..Default::default()
                })
                .await?;

            assert_eq!(db.repositories_all().await?, vec![repo1, repo2]);

            Ok(())
        })
        .await;
    }
}
