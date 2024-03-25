use async_trait::async_trait;
use prbot_models::{PullRequest, PullRequestHandle};
use shaku::{Component, HasComponent, Interface};

use super::GetOrCreateRepositoryInterface;
use crate::{CoreContext, Result};

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait]
pub trait SynchronizePullRequestInterface: Interface {
    async fn run<'a>(&self, ctx: &CoreContext<'a>, pr_handle: &PullRequestHandle) -> Result<()>;
}

#[derive(Component)]
#[shaku(interface = SynchronizePullRequestInterface)]
pub(crate) struct SynchronizePullRequest;

#[async_trait]
impl SynchronizePullRequestInterface for SynchronizePullRequest {
    #[tracing::instrument(skip(self, ctx), fields(pr_handle))]
    async fn run<'a>(&self, ctx: &CoreContext<'a>, pr_handle: &PullRequestHandle) -> Result<()> {
        let get_or_create_uc: &dyn GetOrCreateRepositoryInterface = ctx.core_module.resolve_ref();
        let repo = get_or_create_uc
            .run(ctx, pr_handle.repository_path())
            .await?;

        if ctx
            .db_service
            .pull_requests_get(
                pr_handle.repository_path().owner(),
                pr_handle.repository_path().name(),
                pr_handle.number(),
            )
            .await?
            .is_none()
        {
            ctx.db_service
                .pull_requests_create(
                    PullRequest {
                        number: pr_handle.number(),
                        ..Default::default()
                    }
                    .with_repository(&repo),
                )
                .await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use prbot_database_interface::DbService;
    use prbot_models::{QaStatus, Repository};

    use super::*;
    use crate::context::tests::CoreContextTest;

    #[tokio::test]
    async fn synchronize() {
        let mut ctx = CoreContextTest::new();
        ctx.config.default_needed_reviewers_count = 0;

        SynchronizePullRequest
            .run(&ctx.as_context(), &("me", "test", 1).into())
            .await
            .unwrap();

        assert_eq!(
            ctx.db_service.repositories_all().await.unwrap(),
            vec![Repository {
                id: 1,
                owner: "me".into(),
                name: "test".into(),
                default_needed_reviewers_count: 0,
                default_enable_checks: true,
                default_enable_qa: false,
                ..Default::default()
            }]
        );

        assert_eq!(
            ctx.db_service.pull_requests_all().await.unwrap(),
            vec![PullRequest {
                id: 1,
                number: 1,
                repository_id: 1,
                needed_reviewers_count: 0,
                checks_enabled: true,
                qa_status: QaStatus::Skipped,
                ..Default::default()
            }]
        );
    }
}
