use async_trait::async_trait;
use github_scbot_database_interface::DbService;
use github_scbot_domain_models::PullRequestHandle;
use github_scbot_ghapi_interface::ApiService;
use github_scbot_lock_interface::LockService;

use super::utils::sender::SummaryCommentSender;
use crate::{use_cases::status::PullRequestStatus, Result};

#[mockall::automock]
#[async_trait(?Send)]
pub trait PostSummaryCommentUseCaseInterface {
    async fn run(&self, pr_handle: &PullRequestHandle, pr_status: &PullRequestStatus)
        -> Result<()>;
}

pub struct PostSummaryCommentUseCase<'a> {
    pub api_service: &'a dyn ApiService,
    pub db_service: &'a dyn DbService,
    pub lock_service: &'a dyn LockService,
}

#[async_trait(?Send)]
impl<'a> PostSummaryCommentUseCaseInterface for PostSummaryCommentUseCase<'a> {
    #[tracing::instrument(skip(self), fields(pr_handle))]
    async fn run(
        &self,
        pr_handle: &PullRequestHandle,
        pr_status: &PullRequestStatus,
    ) -> Result<()> {
        SummaryCommentSender::create_or_update(
            self.api_service,
            self.db_service,
            self.lock_service,
            pr_handle,
            pr_status,
        )
        .await
        .map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_database_memory::MemoryDb;
    use github_scbot_domain_models::{PullRequest, Repository};
    use github_scbot_ghapi_interface::MockApiService;
    use github_scbot_lock_interface::{LockInstance, LockStatus, MockLockService};

    use super::*;

    #[tokio::test]
    async fn no_existing_id_lock_ok() {
        let db_service = {
            let svc = MemoryDb::new();
            let repo = svc
                .repositories_create(Repository {
                    owner: "me".into(),
                    name: "test".into(),
                    ..Default::default()
                })
                .await
                .unwrap();
            svc.pull_requests_create(
                PullRequest {
                    number: 1,
                    ..Default::default()
                }
                .with_repository(&repo),
            )
            .await
            .unwrap();

            svc
        };

        let api_service = {
            let mut svc = MockApiService::new();

            svc.expect_comments_post()
                .once()
                .withf(|owner, name, number, body| {
                    owner == "me" && name == "test" && number == &1 && !body.is_empty()
                })
                .return_once(|_, _, _, _| Ok(1));

            svc
        };

        let lock_service = {
            let mut svc = MockLockService::new();

            svc.expect_wait_lock_resource()
                .once()
                .withf(|name, timeout| name == "summary-me-test-1" && timeout == &10000)
                .return_once(|_, _| {
                    Ok(LockStatus::SuccessfullyLocked(LockInstance::new_dummy(
                        "dummy",
                    )))
                });

            svc
        };

        PostSummaryCommentUseCase {
            api_service: &api_service,
            db_service: &db_service,
            lock_service: &lock_service,
        }
        .run(
            &("me", "test", 1).into(),
            &PullRequestStatus {
                ..Default::default()
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn no_existing_id_lock_ko() {
        let db_service = {
            let svc = MemoryDb::new();
            let repo = svc
                .repositories_create(Repository {
                    owner: "me".into(),
                    name: "test".into(),
                    ..Default::default()
                })
                .await
                .unwrap();
            svc.pull_requests_create(
                PullRequest {
                    number: 1,
                    ..Default::default()
                }
                .with_repository(&repo),
            )
            .await
            .unwrap();

            svc
        };

        let api_service = MockApiService::new();

        let lock_service = {
            let mut svc = MockLockService::new();

            svc.expect_wait_lock_resource()
                .once()
                .withf(|name, timeout| name == "summary-me-test-1" && timeout == &10000)
                .return_once(|_, _| Ok(LockStatus::AlreadyLocked));

            svc
        };

        PostSummaryCommentUseCase {
            api_service: &api_service,
            db_service: &db_service,
            lock_service: &lock_service,
        }
        .run(
            &("me", "test", 1).into(),
            &PullRequestStatus {
                ..Default::default()
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn already_existing_id() {
        let db_service = {
            let svc = MemoryDb::new();
            let repo = svc
                .repositories_create(Repository {
                    owner: "me".into(),
                    name: "test".into(),
                    ..Default::default()
                })
                .await
                .unwrap();
            svc.pull_requests_create(
                PullRequest {
                    number: 1,
                    status_comment_id: 1,
                    ..Default::default()
                }
                .with_repository(&repo),
            )
            .await
            .unwrap();

            svc
        };

        let api_service = {
            let mut svc = MockApiService::new();

            svc.expect_comments_update()
                .once()
                .withf(|owner, name, number, body| {
                    owner == "me" && name == "test" && number == &1 && !body.is_empty()
                })
                .return_once(|_, _, _, _| Ok(1));

            svc
        };

        let lock_service = MockLockService::new();

        PostSummaryCommentUseCase {
            api_service: &api_service,
            db_service: &db_service,
            lock_service: &lock_service,
        }
        .run(
            &("me", "test", 1).into(),
            &PullRequestStatus {
                ..Default::default()
            },
        )
        .await
        .unwrap();
    }
}
