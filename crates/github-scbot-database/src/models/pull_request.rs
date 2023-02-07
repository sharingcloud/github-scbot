use github_scbot_core::types::{pulls::GhMergeStrategy, status::QaStatus};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, FromRow, Row};

use crate::{
    fields::{GhMergeStrategyDecode, QaStatusDecode},
    Repository,
};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PullRequest {
    pub id: u64,
    pub repository_id: u64,
    pub number: u64,
    pub qa_status: QaStatus,
    pub needed_reviewers_count: u64,
    pub status_comment_id: u64,
    pub checks_enabled: bool,
    pub automerge: bool,
    pub locked: bool,
    pub strategy_override: Option<GhMergeStrategy>,
}

impl PullRequest {
    pub fn with_repository(mut self, repository: &Repository) -> Self {
        self.repository_id = repository.id;
        self.automerge = repository.default_automerge;
        self.checks_enabled = repository.default_enable_checks;
        self.needed_reviewers_count = repository.default_needed_reviewers_count;
        self.qa_status = if repository.default_enable_qa {
            Default::default()
        } else {
            QaStatus::Skipped
        };
        self
    }
}

impl<'r> FromRow<'r, PgRow> for PullRequest {
    fn from_row(row: &'r PgRow) -> core::result::Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.try_get::<i32, _>("id")? as u64,
            repository_id: row.try_get::<i32, _>("repository_id")? as u64,
            number: row.try_get::<i32, _>("number")? as u64,
            qa_status: *row.try_get::<QaStatusDecode, _>("qa_status")?,
            needed_reviewers_count: row.try_get::<i32, _>("needed_reviewers_count")? as u64,
            status_comment_id: row.try_get::<i32, _>("status_comment_id")? as u64,
            checks_enabled: row.try_get("checks_enabled")?,
            automerge: row.try_get("automerge")?,
            locked: row.try_get("locked")?,
            strategy_override: row
                .try_get::<Option<GhMergeStrategyDecode>, _>("strategy_override")?
                .map(Into::into),
        })
    }
}

#[cfg(test)]
mod new_tests {
    use github_scbot_core::types::{pulls::GhMergeStrategy, status::QaStatus};

    use crate::{utils::db_test_case, DatabaseError, PullRequest, Repository};

    #[actix_rt::test]
    async fn create() {
        db_test_case("pull_request_create", |mut db| async move {
            assert!(matches!(
                db.pull_requests_create(PullRequest {
                    repository_id: 1,
                    ..Default::default()
                })
                .await,
                Err(DatabaseError::UnknownRepositoryId(1))
            ));

            db.repositories_create(Repository {
                owner: "me".into(),
                name: "repo".into(),
                ..Default::default()
            })
            .await?;

            let pr = db
                .pull_requests_create(PullRequest {
                    repository_id: 1,
                    number: 1,
                    ..Default::default()
                })
                .await?;

            assert_eq!(pr.number, 1);

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn update() {
        db_test_case("pull_request_update", |mut db| async move {
            assert!(matches!(
                db.pull_requests_update(PullRequest {
                    id: 1,
                    repository_id: 1,
                    ..Default::default()
                })
                .await,
                Err(DatabaseError::UnknownRepositoryId(1))
            ));

            let repo = db
                .repositories_create(Repository {
                    owner: "me".into(),
                    name: "repo".into(),
                    ..Default::default()
                })
                .await?;

            assert!(matches!(
                db.pull_requests_update(PullRequest {
                    id: 1,
                    repository_id: repo.id,
                    ..Default::default()
                })
                .await,
                Err(DatabaseError::UnknownPullRequest(_, _))
            ));

            let pr = db
                .pull_requests_create(PullRequest {
                    repository_id: repo.id,
                    number: 1,
                    ..Default::default()
                })
                .await?;

            let pr = db
                .pull_requests_update(PullRequest {
                    id: pr.id,
                    repository_id: pr.repository_id,
                    number: 1,
                    locked: true,
                    ..Default::default()
                })
                .await?;

            assert!(pr.locked);

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn get() {
        db_test_case("pull_request_get", |mut db| async move {
            assert_eq!(db.pull_requests_get("me", "repo", 1).await?, None);

            let repo = db
                .repositories_create(Repository {
                    owner: "me".into(),
                    name: "repo".into(),
                    ..Default::default()
                })
                .await?;

            let pr = db
                .pull_requests_create(PullRequest {
                    repository_id: repo.id,
                    number: 1,
                    ..Default::default()
                })
                .await?;

            let get_pr = db.pull_requests_get("me", "repo", 1).await?;
            assert_eq!(get_pr, Some(pr));

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn get_from_id() {
        db_test_case("pull_request_get_from_id", |mut db| async move {
            assert_eq!(db.pull_requests_get_from_id(1).await?, None);

            let repo = db
                .repositories_create(Repository {
                    owner: "me".into(),
                    name: "repo".into(),
                    ..Default::default()
                })
                .await?;

            let pr = db
                .pull_requests_create(PullRequest {
                    repository_id: repo.id,
                    number: 1,
                    ..Default::default()
                })
                .await?;

            let get_pr = db.pull_requests_get_from_id(pr.id).await?;
            assert_eq!(get_pr, Some(pr));

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn delete() {
        db_test_case("pull_request_delete", |mut db| async move {
            assert!(!db.pull_requests_delete("me", "repo", 1).await?);

            let repo = db
                .repositories_create(Repository {
                    owner: "me".into(),
                    name: "repo".into(),
                    ..Default::default()
                })
                .await?;

            db.pull_requests_create(PullRequest {
                repository_id: repo.id,
                number: 1,
                ..Default::default()
            })
            .await?;

            assert!(db.pull_requests_delete("me", "repo", 1).await?);
            assert_eq!(db.pull_requests_get("me", "repo", 1).await?, None);

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn list() {
        db_test_case("pull_request_list", |mut db| async move {
            assert_eq!(db.pull_requests_list("me", "repo").await?, vec![]);

            let repo = db
                .repositories_create(Repository {
                    owner: "me".into(),
                    name: "repo".into(),
                    ..Default::default()
                })
                .await?;

            let pr1 = db
                .pull_requests_create(PullRequest {
                    repository_id: repo.id,
                    number: 1,
                    ..Default::default()
                })
                .await?;

            let pr2 = db
                .pull_requests_create(PullRequest {
                    repository_id: repo.id,
                    number: 2,
                    ..Default::default()
                })
                .await?;

            assert_eq!(db.pull_requests_list("me", "repo").await?, vec![pr1, pr2]);

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn all() {
        db_test_case("pull_request_all", |mut db| async move {
            assert_eq!(db.pull_requests_all().await?, vec![]);

            let repo = db
                .repositories_create(Repository {
                    owner: "me".into(),
                    name: "repo".into(),
                    ..Default::default()
                })
                .await?;

            let pr1 = db
                .pull_requests_create(PullRequest {
                    repository_id: repo.id,
                    number: 1,
                    ..Default::default()
                })
                .await?;

            let pr2 = db
                .pull_requests_create(PullRequest {
                    repository_id: repo.id,
                    number: 2,
                    ..Default::default()
                })
                .await?;

            assert_eq!(db.pull_requests_all().await?, vec![pr1, pr2]);

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn set_qa_status() {
        db_test_case("pull_request_set_qa_status", |mut db| async move {
            assert!(matches!(
                db.pull_requests_set_qa_status("me", "repo", 1, QaStatus::Skipped)
                    .await,
                Err(DatabaseError::UnknownRepository(_))
            ));

            let repo = db
                .repositories_create(Repository {
                    owner: "me".into(),
                    name: "repo".into(),
                    ..Default::default()
                })
                .await?;

            assert!(matches!(
                db.pull_requests_set_qa_status("me", "repo", 1, QaStatus::Skipped)
                    .await,
                Err(DatabaseError::UnknownPullRequest(_, _))
            ));

            db.pull_requests_create(PullRequest {
                repository_id: repo.id,
                number: 1,
                qa_status: QaStatus::Pass,
                ..Default::default()
            })
            .await?;

            let pr = db
                .pull_requests_set_qa_status("me", "repo", 1, QaStatus::Fail)
                .await?;
            assert_eq!(pr.qa_status, QaStatus::Fail);

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn set_needed_reviewers_count() {
        db_test_case(
            "pull_request_set_needed_reviewers_count",
            |mut db| async move {
                assert!(matches!(
                    db.pull_requests_set_needed_reviewers_count("me", "repo", 1, 1)
                        .await,
                    Err(DatabaseError::UnknownRepository(_))
                ));

                let repo = db
                    .repositories_create(Repository {
                        owner: "me".into(),
                        name: "repo".into(),
                        ..Default::default()
                    })
                    .await?;

                assert!(matches!(
                    db.pull_requests_set_needed_reviewers_count("me", "repo", 1, 1)
                        .await,
                    Err(DatabaseError::UnknownPullRequest(_, _))
                ));

                db.pull_requests_create(PullRequest {
                    repository_id: repo.id,
                    number: 1,
                    needed_reviewers_count: 0,
                    ..Default::default()
                })
                .await?;

                let pr = db
                    .pull_requests_set_needed_reviewers_count("me", "repo", 1, 1)
                    .await?;
                assert_eq!(pr.needed_reviewers_count, 1);

                Ok(())
            },
        )
        .await;
    }

    #[actix_rt::test]
    async fn set_status_comment_id() {
        db_test_case("pull_request_set_status_comment_id", |mut db| async move {
            assert!(matches!(
                db.pull_requests_set_status_comment_id("me", "repo", 1, 1)
                    .await,
                Err(DatabaseError::UnknownRepository(_))
            ));

            let repo = db
                .repositories_create(Repository {
                    owner: "me".into(),
                    name: "repo".into(),
                    ..Default::default()
                })
                .await?;

            assert!(matches!(
                db.pull_requests_set_status_comment_id("me", "repo", 1, 1)
                    .await,
                Err(DatabaseError::UnknownPullRequest(_, _))
            ));

            db.pull_requests_create(PullRequest {
                repository_id: repo.id,
                number: 1,
                status_comment_id: 0,
                ..Default::default()
            })
            .await?;

            let pr = db
                .pull_requests_set_status_comment_id("me", "repo", 1, 1)
                .await?;
            assert_eq!(pr.status_comment_id, 1);

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn set_checks_enabled() {
        db_test_case("pull_request_set_checks_enabled", |mut db| async move {
            assert!(matches!(
                db.pull_requests_set_checks_enabled("me", "repo", 1, true)
                    .await,
                Err(DatabaseError::UnknownRepository(_))
            ));

            let repo = db
                .repositories_create(Repository {
                    owner: "me".into(),
                    name: "repo".into(),
                    ..Default::default()
                })
                .await?;

            assert!(matches!(
                db.pull_requests_set_checks_enabled("me", "repo", 1, true)
                    .await,
                Err(DatabaseError::UnknownPullRequest(_, _))
            ));

            db.pull_requests_create(PullRequest {
                repository_id: repo.id,
                number: 1,
                checks_enabled: false,
                ..Default::default()
            })
            .await?;

            let pr = db
                .pull_requests_set_checks_enabled("me", "repo", 1, true)
                .await?;
            assert!(pr.checks_enabled);

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn set_automerge() {
        db_test_case("pull_request_set_automerge", |mut db| async move {
            assert!(matches!(
                db.pull_requests_set_automerge("me", "repo", 1, true).await,
                Err(DatabaseError::UnknownRepository(_))
            ));

            let repo = db
                .repositories_create(Repository {
                    owner: "me".into(),
                    name: "repo".into(),
                    ..Default::default()
                })
                .await?;

            assert!(matches!(
                db.pull_requests_set_automerge("me", "repo", 1, true).await,
                Err(DatabaseError::UnknownPullRequest(_, _))
            ));

            db.pull_requests_create(PullRequest {
                repository_id: repo.id,
                number: 1,
                automerge: false,
                ..Default::default()
            })
            .await?;

            let pr = db
                .pull_requests_set_automerge("me", "repo", 1, true)
                .await?;
            assert!(pr.automerge);

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn set_locked() {
        db_test_case("pull_request_set_locked", |mut db| async move {
            assert!(matches!(
                db.pull_requests_set_locked("me", "repo", 1, true).await,
                Err(DatabaseError::UnknownRepository(_))
            ));

            let repo = db
                .repositories_create(Repository {
                    owner: "me".into(),
                    name: "repo".into(),
                    ..Default::default()
                })
                .await?;

            assert!(matches!(
                db.pull_requests_set_locked("me", "repo", 1, true).await,
                Err(DatabaseError::UnknownPullRequest(_, _))
            ));

            db.pull_requests_create(PullRequest {
                repository_id: repo.id,
                number: 1,
                locked: false,
                ..Default::default()
            })
            .await?;

            let pr = db.pull_requests_set_locked("me", "repo", 1, true).await?;
            assert!(pr.locked);

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn set_strategy_override() {
        db_test_case("pull_request_set_strategy_override", |mut db| async move {
            assert!(matches!(
                db.pull_requests_set_strategy_override(
                    "me",
                    "repo",
                    1,
                    Some(GhMergeStrategy::Squash)
                )
                .await,
                Err(DatabaseError::UnknownRepository(_))
            ));

            let repo = db
                .repositories_create(Repository {
                    owner: "me".into(),
                    name: "repo".into(),
                    ..Default::default()
                })
                .await?;

            assert!(matches!(
                db.pull_requests_set_strategy_override(
                    "me",
                    "repo",
                    1,
                    Some(GhMergeStrategy::Squash)
                )
                .await,
                Err(DatabaseError::UnknownPullRequest(_, _))
            ));

            db.pull_requests_create(PullRequest {
                repository_id: repo.id,
                number: 1,
                strategy_override: None,
                ..Default::default()
            })
            .await?;

            let pr = db
                .pull_requests_set_strategy_override("me", "repo", 1, Some(GhMergeStrategy::Squash))
                .await?;
            assert_eq!(pr.strategy_override, Some(GhMergeStrategy::Squash));

            Ok(())
        })
        .await;
    }
}
