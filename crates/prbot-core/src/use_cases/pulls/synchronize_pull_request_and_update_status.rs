use async_trait::async_trait;
use prbot_models::PullRequestHandle;
use shaku::{Component, HasComponent, Interface};

use super::SynchronizePullRequestInterface;
use crate::{use_cases::status::UpdatePullRequestStatusInterface, CoreContext, Result};

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait]
pub trait SynchronizePullRequestAndUpdateStatusInterface: Interface {
    async fn run<'a>(&self, ctx: &CoreContext<'a>, pr_handle: &PullRequestHandle) -> Result<()>;
}

#[derive(Component)]
#[shaku(interface = SynchronizePullRequestAndUpdateStatusInterface)]
pub(crate) struct SynchronizePullRequestAndUpdateStatus;

#[async_trait]
impl SynchronizePullRequestAndUpdateStatusInterface for SynchronizePullRequestAndUpdateStatus {
    #[tracing::instrument(skip(self, ctx), fields(pr_handle))]
    async fn run<'a>(&self, ctx: &CoreContext<'a>, pr_handle: &PullRequestHandle) -> Result<()> {
        let synchronize_pull_request: &dyn SynchronizePullRequestInterface =
            ctx.core_module.resolve_ref();
        synchronize_pull_request.run(ctx, pr_handle).await?;

        let upstream_pr = ctx
            .api_service
            .pulls_get(
                pr_handle.repository_path().owner(),
                pr_handle.repository_path().name(),
                pr_handle.number(),
            )
            .await?;

        let update_pull_request_status: &dyn UpdatePullRequestStatusInterface =
            ctx.core_module.resolve_ref();
        update_pull_request_status
            .run(ctx, pr_handle, &upstream_pr)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use prbot_ghapi_interface::types::GhPullRequest;

    use super::*;
    use crate::{
        context::tests::CoreContextTest,
        use_cases::{
            pulls::MockSynchronizePullRequestInterface,
            status::MockUpdatePullRequestStatusInterface,
        },
        CoreModule,
    };

    #[tokio::test]
    async fn run() {
        let mut ctx = CoreContextTest::new();
        let mut synchronize_pull_request = MockSynchronizePullRequestInterface::new();
        let mut update_pull_request_status = MockUpdatePullRequestStatusInterface::new();

        ctx.api_service
            .expect_pulls_get()
            .once()
            .withf(|owner, name, number| owner == "me" && name == "test" && number == &1)
            .return_once(|_, _, _| {
                Ok(GhPullRequest {
                    number: 1,
                    ..Default::default()
                })
            });

        synchronize_pull_request
            .expect_run()
            .once()
            .withf(|_, pr_handle| pr_handle == &("me", "test", 1).into())
            .return_once(|_, _| Ok(()));

        update_pull_request_status
            .expect_run()
            .once()
            .withf(|_, pr_handle, upstream_pr| {
                pr_handle == &("me", "test", 1).into() && upstream_pr.number == 1
            })
            .return_once(|_, _, _| Ok(()));

        ctx.core_module = CoreModule::builder()
            .with_component_override::<dyn SynchronizePullRequestInterface>(Box::new(
                synchronize_pull_request,
            ))
            .with_component_override::<dyn UpdatePullRequestStatusInterface>(Box::new(
                update_pull_request_status,
            ))
            .build();

        SynchronizePullRequestAndUpdateStatus
            .run(&ctx.as_context(), &("me", "test", 1).into())
            .await
            .unwrap();
    }
}
