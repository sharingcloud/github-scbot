use async_trait::async_trait;
use prbot_ghapi_interface::types::GhCommitStatusState;
use prbot_models::PullRequestHandle;
use shaku::{Component, HasComponent, Interface};

use super::utils::VALIDATION_STATUS_MESSAGE;
use crate::{use_cases::summary::DeleteSummaryCommentInterface, CoreContext, Result};

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait]
pub trait DisablePullRequestStatusInterface: Interface {
    async fn run<'a>(&self, ctx: &CoreContext<'a>, pr_handle: &PullRequestHandle) -> Result<()>;
}

#[derive(Component)]
#[shaku(interface = DisablePullRequestStatusInterface)]
pub(crate) struct DisablePullRequestStatus;

#[async_trait]
impl DisablePullRequestStatusInterface for DisablePullRequestStatus {
    #[tracing::instrument(skip(self, ctx), fields(pr_handle))]
    async fn run<'b>(&self, ctx: &CoreContext<'b>, pr_handle: &PullRequestHandle) -> Result<()> {
        let sha = ctx
            .api_service
            .pulls_get(
                pr_handle.repository_path().owner(),
                pr_handle.repository_path().name(),
                pr_handle.number(),
            )
            .await?
            .head
            .sha;

        ctx.api_service
            .commit_statuses_update(
                pr_handle.repository_path().owner(),
                pr_handle.repository_path().name(),
                &sha,
                GhCommitStatusState::Success,
                VALIDATION_STATUS_MESSAGE,
                "Bot disabled.",
            )
            .await?;

        let delete_summary_comment: &dyn DeleteSummaryCommentInterface =
            ctx.core_module.resolve_ref();
        delete_summary_comment.run(ctx, pr_handle).await
    }
}

#[cfg(test)]
mod tests {
    use prbot_ghapi_interface::{
        types::{GhBranch, GhPullRequest},
        MockApiService,
    };

    use super::*;
    use crate::{
        context::tests::CoreContextTest, use_cases::summary::MockDeleteSummaryCommentInterface,
        CoreModule,
    };

    #[tokio::test]
    async fn run() {
        let mut ctx = CoreContextTest::new();
        ctx.api_service = {
            let mut api_service = MockApiService::new();
            api_service
                .expect_pulls_get()
                .once()
                .withf(|owner, name, number| owner == "me" && name == "test" && number == &1)
                .return_once(|_, _, _| {
                    Ok(GhPullRequest {
                        number: 1,
                        head: GhBranch {
                            sha: "abcdef".into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                });

            api_service
                .expect_commit_statuses_update()
                .once()
                .withf(|owner, name, sha, status, title, body| {
                    owner == "me"
                        && name == "test"
                        && sha == "abcdef"
                        && *status == GhCommitStatusState::Success
                        && title == VALIDATION_STATUS_MESSAGE
                        && body == "Bot disabled."
                })
                .return_once(|_, _, _, _, _, _| Ok(()));

            api_service
        };

        let delete_summary_comment = {
            let mut delete_summary_comment = MockDeleteSummaryCommentInterface::new();
            delete_summary_comment
                .expect_run()
                .once()
                .withf(|_, pr_handle| pr_handle == &("me", "test", 1).into())
                .return_once(|_, _| Ok(()));
            delete_summary_comment
        };

        ctx.core_module = CoreModule::builder()
            .with_component_override::<dyn DeleteSummaryCommentInterface>(Box::new(
                delete_summary_comment,
            ))
            .build();

        DisablePullRequestStatus
            .run(&ctx.as_context(), &("me", "test", 1).into())
            .await
            .unwrap()
    }
}
