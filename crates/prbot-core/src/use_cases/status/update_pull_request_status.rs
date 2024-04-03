use async_trait::async_trait;
use prbot_ghapi_interface::types::GhPullRequest;
use prbot_models::PullRequestHandle;
use shaku::{Component, HasComponent, Interface};

use super::{
    build_pull_request_status::BuildPullRequestStatusInterface, CreateOrUpdateCommitStatusInterface,
};
use crate::{
    use_cases::{
        pulls::{
            try_merge_pull_request_from_status::TryMergePullRequestState,
            AutomergePullRequestInterface, UpdateStepLabelFromStatusInterface,
        },
        summary::PostSummaryCommentInterface,
    },
    CoreContext, Result,
};

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait]
pub trait UpdatePullRequestStatusInterface: Interface {
    async fn run<'b>(
        &self,
        ctx: &CoreContext<'b>,
        pr_handle: &PullRequestHandle,
        upstream_pr: &GhPullRequest,
    ) -> Result<()>;
}

#[derive(Component)]
#[shaku(interface = UpdatePullRequestStatusInterface)]
pub(crate) struct UpdatePullRequestStatus;

#[async_trait]
impl UpdatePullRequestStatusInterface for UpdatePullRequestStatus {
    #[tracing::instrument(
        skip_all,
        fields(
            pr_handle,
            head_sha = %upstream_pr.head.sha
        )
    )]
    async fn run<'b>(
        &self,
        ctx: &CoreContext<'b>,
        pr_handle: &PullRequestHandle,
        upstream_pr: &GhPullRequest,
    ) -> Result<()> {
        // Build PR status.
        let build_pull_request_status: &dyn BuildPullRequestStatusInterface =
            ctx.core_module.resolve_ref();
        let pr_status = build_pull_request_status
            .run(ctx, pr_handle, upstream_pr)
            .await?;

        // Update step label.
        let update_step_label: &dyn UpdateStepLabelFromStatusInterface =
            ctx.core_module.resolve_ref();
        update_step_label.run(ctx, pr_handle, &pr_status).await?;

        // Update summary comment.
        let post_summary_comment: &dyn PostSummaryCommentInterface = ctx.core_module.resolve_ref();
        post_summary_comment.run(ctx, pr_handle, &pr_status).await?;

        // Create or update status.
        let create_or_update_commit_status: &dyn CreateOrUpdateCommitStatusInterface =
            ctx.core_module.resolve_ref();
        create_or_update_commit_status
            .run(ctx, pr_handle, &pr_status, upstream_pr)
            .await?;

        let pr_model = ctx
            .db_service
            .pull_requests_get(
                pr_handle.repository_path().owner(),
                pr_handle.repository_path().name(),
                pr_handle.number(),
            )
            .await?
            .unwrap();

        if pr_model.automerge {
            let automerge_uc: &dyn AutomergePullRequestInterface = ctx.core_module.resolve_ref();
            let result = automerge_uc
                .run(ctx, pr_handle, upstream_pr, &pr_status)
                .await?;
            if result == TryMergePullRequestState::Error {
                // Disable automerge.
                ctx.db_service
                    .pull_requests_set_automerge(
                        pr_handle.repository_path().owner(),
                        pr_handle.repository_path().name(),
                        pr_handle.number(),
                        false,
                    )
                    .await?;

                // Update summary comment.
                let post_summary_comment: &dyn PostSummaryCommentInterface =
                    ctx.core_module.resolve_ref();
                post_summary_comment.run(ctx, pr_handle, &pr_status).await?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use build_pull_request_status::MockBuildPullRequestStatusInterface;
    use prbot_database_interface::DbService;
    use prbot_models::{MergeStrategy, PullRequest, Repository, StepLabel};
    use shaku::ModuleBuilder;

    use super::*;
    use crate::{
        context::tests::CoreContextTest,
        use_cases::{
            pulls::{MockAutomergePullRequestInterface, MockUpdateStepLabelFromStatusInterface},
            status::{
                build_pull_request_status, MockCreateOrUpdateCommitStatusInterface,
                PullRequestStatus,
            },
            summary::MockPostSummaryCommentInterface,
        },
        CoreModule,
    };

    struct Arrange {
        ctx: CoreContextTest,
    }

    impl Arrange {
        async fn check(&self) {
            UpdatePullRequestStatus
                .run(
                    &self.ctx.as_context(),
                    &("owner", "name", 1).into(),
                    &GhPullRequest {
                        ..Default::default()
                    },
                )
                .await
                .unwrap();
        }

        async fn setup_core_module(&mut self, f: impl Fn(ModuleBuilder<CoreModule>) -> CoreModule) {
            let mut build_mock = MockBuildPullRequestStatusInterface::new();
            build_mock.expect_run().return_once(|_, _, _| {
                Ok(PullRequestStatus {
                    ..Default::default()
                })
            });

            let mut update_step_label = MockUpdateStepLabelFromStatusInterface::new();
            update_step_label
                .expect_run()
                .return_once(|_, _, _| Ok(StepLabel::Wip));

            let mut post_summary_comment = MockPostSummaryCommentInterface::new();
            post_summary_comment
                .expect_run()
                .return_once(|_, _, _| Ok(()));

            let mut create_or_update_commit_status = MockCreateOrUpdateCommitStatusInterface::new();
            create_or_update_commit_status
                .expect_run()
                .return_once(|_, _, _, _| Ok(()));

            let builder = CoreModule::builder()
                .with_component_override::<dyn BuildPullRequestStatusInterface>(Box::new(
                    build_mock,
                ))
                .with_component_override::<dyn UpdateStepLabelFromStatusInterface>(Box::new(
                    update_step_label,
                ))
                .with_component_override::<dyn PostSummaryCommentInterface>(Box::new(
                    post_summary_comment,
                ))
                .with_component_override::<dyn CreateOrUpdateCommitStatusInterface>(Box::new(
                    create_or_update_commit_status,
                ));

            self.ctx.core_module = f(builder);
        }
    }

    async fn arrange(automerge: bool) -> Arrange {
        let ctx = CoreContextTest::new();

        let repo = ctx
            .db_service
            .repositories_create(Repository {
                owner: "owner".into(),
                name: "name".into(),
                ..Default::default()
            })
            .await
            .unwrap();

        ctx.db_service
            .pull_requests_create(PullRequest {
                repository_id: repo.id,
                number: 1,
                automerge,
                ..Default::default()
            })
            .await
            .unwrap();

        Arrange { ctx }
    }

    #[tokio::test]
    async fn no_automerge() {
        let mut arrange = arrange(false).await;
        arrange.setup_core_module(|builder| builder.build()).await;
        arrange.check().await;
    }

    #[tokio::test]
    async fn automerge_failure() {
        let mut arrange = arrange(true).await;
        arrange
            .setup_core_module(|builder| {
                let mut automerge = MockAutomergePullRequestInterface::new();
                automerge
                    .expect_run()
                    .return_once(|_, _, _, _| Ok(TryMergePullRequestState::Error));

                builder.build()
            })
            .await;

        arrange.check().await;
    }

    #[tokio::test]
    async fn automerge_success() {
        let mut arrange = arrange(true).await;
        arrange
            .setup_core_module(|builder| {
                let mut automerge = MockAutomergePullRequestInterface::new();
                automerge.expect_run().return_once(|_, _, _, _| {
                    Ok(TryMergePullRequestState::Success(MergeStrategy::Merge))
                });

                builder.build()
            })
            .await;

        arrange.check().await;
    }
}
