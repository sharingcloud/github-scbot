use async_trait::async_trait;
use prbot_models::PullRequestHandle;
use shaku::{Component, Interface};

use super::utils::sender::SummaryCommentSender;
use crate::{CoreContext, Result};

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait]
pub trait DeleteSummaryCommentInterface: Interface {
    async fn run<'a>(&self, ctx: &CoreContext<'a>, pr_handle: &'a PullRequestHandle) -> Result<()>;
}

#[derive(Component)]
#[shaku(interface = DeleteSummaryCommentInterface)]
pub(crate) struct DeleteSummaryComment;

#[async_trait]
impl DeleteSummaryCommentInterface for DeleteSummaryComment {
    #[tracing::instrument(skip(self, ctx), fields(pr_handle))]
    async fn run<'a>(&self, ctx: &CoreContext<'a>, pr_handle: &'a PullRequestHandle) -> Result<()> {
        SummaryCommentSender::delete(ctx, pr_handle)
            .await
            .map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use prbot_database_interface::DbService;
    use prbot_database_memory::MemoryDb;
    use prbot_ghapi_interface::MockApiService;
    use prbot_models::{PullRequest, Repository};

    use super::*;
    use crate::context::tests::CoreContextTest;

    #[tokio::test]
    async fn run_no_existing_id() {
        let ctx = CoreContextTest::new();

        DeleteSummaryComment
            .run(&ctx.as_context(), &("me", "test", 1).into())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn run_existing_id() {
        let mut ctx = CoreContextTest::new();

        ctx.api_service = {
            let mut svc = MockApiService::new();
            svc.expect_comments_delete()
                .once()
                .withf(|owner, name, comment_id| {
                    owner == "me" && name == "test" && comment_id == &1
                })
                .return_once(|_, _, _| Ok(()));

            svc
        };

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

        DeleteSummaryComment
            .run(&ctx.as_context(), &("me", "test", 1).into())
            .await
            .unwrap();
    }
}
