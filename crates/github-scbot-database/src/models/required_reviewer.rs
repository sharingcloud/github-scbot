use github_scbot_macros::SCGetter;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, FromRow, Row};

use crate::PullRequest;

#[derive(
    SCGetter, Debug, Clone, Default, derive_builder::Builder, Serialize, Deserialize, PartialEq, Eq,
)]
#[builder(default, setter(into))]
pub struct RequiredReviewer {
    #[get]
    pub(crate) pull_request_id: u64,
    #[get_deref]
    pub(crate) username: String,
}

impl RequiredReviewer {
    pub fn builder() -> RequiredReviewerBuilder {
        RequiredReviewerBuilder::default()
    }

    pub fn set_pull_request_id(&mut self, id: u64) {
        self.pull_request_id = id;
    }
}

impl RequiredReviewerBuilder {
    pub fn with_pull_request(&mut self, pull_request: &PullRequest) -> &mut Self {
        self.pull_request_id = Some(pull_request.id());
        self
    }
}

impl<'r> FromRow<'r, PgRow> for RequiredReviewer {
    fn from_row(row: &'r PgRow) -> core::result::Result<Self, sqlx::Error> {
        Ok(Self {
            pull_request_id: row.try_get::<i32, _>("pull_request_id")? as u64,
            username: row.try_get("username")?,
        })
    }
}

#[cfg(test)]
mod new_tests {
    use crate::{utils::db_test_case, PullRequest, Repository, RequiredReviewer};

    #[actix_rt::test]
    async fn create() {
        db_test_case("required_reviewer_create", |mut db| async move {
            let repo = db
                .repositories_create(Repository::builder().owner("me").name("repo").build()?)
                .await?;

            let pr = db
                .pull_requests_create(
                    PullRequest::builder()
                        .repository_id(repo.id())
                        .number(1u64)
                        .build()?,
                )
                .await?;

            let r = db
                .required_reviewers_create(
                    RequiredReviewer::builder()
                        .pull_request_id(pr.id())
                        .username("me")
                        .build()?,
                )
                .await?;

            assert_eq!(r.pull_request_id(), pr.id());
            assert_eq!(r.username(), "me");

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn list() {
        db_test_case("required_reviewer_list", |mut db| async move {
            let repo = db
                .repositories_create(Repository::builder().owner("me").name("repo").build()?)
                .await?;
            let pr = db
                .pull_requests_create(
                    PullRequest::builder()
                        .repository_id(repo.id())
                        .number(1u64)
                        .build()?,
                )
                .await?;

            let r1 = db
                .required_reviewers_create(
                    RequiredReviewer::builder()
                        .pull_request_id(pr.id())
                        .username("me")
                        .build()?,
                )
                .await?;
            let r2 = db
                .required_reviewers_create(
                    RequiredReviewer::builder()
                        .pull_request_id(pr.id())
                        .username("her")
                        .build()?,
                )
                .await?;

            assert_eq!(
                db.required_reviewers_list("me", "repo", 1).await?,
                vec![r2, r1]
            );

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn all() {
        db_test_case("required_reviewer_all", |mut db| async move {
            let repo = db
                .repositories_create(Repository::builder().owner("me").name("repo").build()?)
                .await?;
            let pr1 = db
                .pull_requests_create(
                    PullRequest::builder()
                        .repository_id(repo.id())
                        .number(1u64)
                        .build()?,
                )
                .await?;
            let pr2 = db
                .pull_requests_create(
                    PullRequest::builder()
                        .repository_id(repo.id())
                        .number(2u64)
                        .build()?,
                )
                .await?;

            let r1 = db
                .required_reviewers_create(
                    RequiredReviewer::builder()
                        .pull_request_id(pr1.id())
                        .username("me")
                        .build()?,
                )
                .await?;
            let r2 = db
                .required_reviewers_create(
                    RequiredReviewer::builder()
                        .pull_request_id(pr1.id())
                        .username("her")
                        .build()?,
                )
                .await?;
            let r3 = db
                .required_reviewers_create(
                    RequiredReviewer::builder()
                        .pull_request_id(pr2.id())
                        .username("me")
                        .build()?,
                )
                .await?;

            assert_eq!(db.required_reviewers_all().await?, vec![r2, r1, r3]);

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn get() {
        db_test_case("required_reviewer_get", |mut db| async move {
            let repo = db
                .repositories_create(Repository::builder().owner("me").name("repo").build()?)
                .await?;
            let pr = db
                .pull_requests_create(
                    PullRequest::builder()
                        .repository_id(repo.id())
                        .number(1u64)
                        .build()?,
                )
                .await?;

            assert_eq!(
                db.required_reviewers_get("me", "repo", 1, "me").await?,
                None
            );

            let r = db
                .required_reviewers_create(
                    RequiredReviewer::builder()
                        .pull_request_id(pr.id())
                        .username("me")
                        .build()?,
                )
                .await?;

            assert_eq!(
                db.required_reviewers_get("me", "repo", 1, "me").await?,
                Some(r)
            );

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn delete() {
        db_test_case("required_reviewer_delete", |mut db| async move {
            let repo = db
                .repositories_create(Repository::builder().owner("me").name("repo").build()?)
                .await?;
            let pr = db
                .pull_requests_create(
                    PullRequest::builder()
                        .repository_id(repo.id())
                        .number(1u64)
                        .build()?,
                )
                .await?;

            assert_eq!(
                db.required_reviewers_delete("me", "repo", 1, "me").await?,
                false
            );

            db.required_reviewers_create(
                RequiredReviewer::builder()
                    .pull_request_id(pr.id())
                    .username("me")
                    .build()?,
            )
            .await?;

            assert_eq!(
                db.required_reviewers_delete("me", "repo", 1, "me").await?,
                true
            );
            assert_eq!(
                db.required_reviewers_get("me", "repo", 1, "me").await?,
                None
            );

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn cascade_pull_request() {
        db_test_case(
            "required_reviewer_cascade_pull_request",
            |mut db| async move {
                let repo = db
                    .repositories_create(Repository::builder().owner("me").name("repo").build()?)
                    .await?;
                let pr = db
                    .pull_requests_create(
                        PullRequest::builder()
                            .repository_id(repo.id())
                            .number(1u64)
                            .build()?,
                    )
                    .await?;

                db.required_reviewers_create(
                    RequiredReviewer::builder()
                        .pull_request_id(pr.id())
                        .username("me")
                        .build()?,
                )
                .await?;

                db.pull_requests_delete("me", "repo", 1).await?;
                assert_eq!(db.required_reviewers_all().await?, vec![]);

                Ok(())
            },
        )
        .await;
    }

    #[actix_rt::test]
    async fn cascade_repository() {
        db_test_case(
            "required_reviewer_cascade_repository",
            |mut db| async move {
                let repo = db
                    .repositories_create(Repository::builder().owner("me").name("repo").build()?)
                    .await?;
                let pr = db
                    .pull_requests_create(
                        PullRequest::builder()
                            .repository_id(repo.id())
                            .number(1u64)
                            .build()?,
                    )
                    .await?;

                db.required_reviewers_create(
                    RequiredReviewer::builder()
                        .pull_request_id(pr.id())
                        .username("me")
                        .build()?,
                )
                .await?;

                db.repositories_delete("me", "repo").await?;
                assert_eq!(db.required_reviewers_all().await?, vec![]);

                Ok(())
            },
        )
        .await;
    }
}
