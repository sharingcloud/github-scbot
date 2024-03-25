use async_trait::async_trait;
use prbot_models::PullRequestHandle;
use shaku::{Component, Interface};

use super::utils::sender::SummaryCommentSender;
use crate::{use_cases::status::PullRequestStatus, CoreContext, Result};

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait]
pub trait PostSummaryCommentInterface: Interface {
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        pr_handle: &'a PullRequestHandle,
        pr_status: &'a PullRequestStatus,
    ) -> Result<()>;
}

#[derive(Component)]
#[shaku(interface = PostSummaryCommentInterface)]
pub(crate) struct PostSummaryComment;

#[async_trait]
impl PostSummaryCommentInterface for PostSummaryComment {
    #[tracing::instrument(skip(self, ctx), fields(pr_handle))]
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        pr_handle: &'a PullRequestHandle,
        pr_status: &'a PullRequestStatus,
    ) -> Result<()> {
        SummaryCommentSender::create_or_update(ctx, pr_handle, pr_status)
            .await
            .map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use prbot_database_interface::DbService;
    use prbot_database_memory::MemoryDb;
    use prbot_ghapi_interface::MockApiService;
    use prbot_lock_interface::{LockInstance, LockStatus, MockLockService};
    use prbot_models::{PullRequest, Repository};

    use super::*;
    use crate::context::tests::CoreContextTest;

    #[tokio::test]
    async fn no_existing_id_lock_ok() {
        let mut ctx = CoreContextTest::new();
        ctx.db_service = {
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

        ctx.api_service = {
            let mut svc = MockApiService::new();

            svc.expect_comments_post()
                .once()
                .withf(|owner, name, number, body| {
                    owner == "me" && name == "test" && number == &1 && !body.is_empty()
                })
                .return_once(|_, _, _, _| Ok(1));

            svc
        };

        ctx.lock_service = {
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

        PostSummaryComment
            .run(
                &ctx.as_context(),
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
        let mut ctx = CoreContextTest::new();
        ctx.db_service = {
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

        ctx.api_service = MockApiService::new();

        ctx.lock_service = {
            let mut svc = MockLockService::new();

            svc.expect_wait_lock_resource()
                .once()
                .withf(|name, timeout| name == "summary-me-test-1" && timeout == &10000)
                .return_once(|_, _| Ok(LockStatus::AlreadyLocked));

            svc
        };

        PostSummaryComment
            .run(
                &ctx.as_context(),
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
        let mut ctx = CoreContextTest::new();
        ctx.db_service = {
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

        ctx.api_service = {
            let mut svc = MockApiService::new();

            svc.expect_comments_update()
                .once()
                .withf(|owner, name, number, body| {
                    owner == "me" && name == "test" && number == &1 && !body.is_empty()
                })
                .return_once(|_, _, _, _| Ok(1));

            svc
        };

        ctx.lock_service = MockLockService::new();

        PostSummaryComment
            .run(
                &ctx.as_context(),
                &("me", "test", 1).into(),
                &PullRequestStatus {
                    ..Default::default()
                },
            )
            .await
            .unwrap();
    }
}
