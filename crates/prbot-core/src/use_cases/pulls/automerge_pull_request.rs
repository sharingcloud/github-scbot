use async_trait::async_trait;
use prbot_ghapi_interface::{comments::CommentApi, types::GhPullRequest};
use prbot_models::PullRequestHandle;
use shaku::{Component, HasComponent, Interface};

use super::{
    try_merge_pull_request_from_status::TryMergePullRequestState,
    TryMergePullRequestFromStatusInterface,
};
use crate::{use_cases::status::PullRequestStatus, CoreContext, Result};

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait]
pub trait AutomergePullRequestInterface: Interface {
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        pr_handle: &PullRequestHandle,
        upstream_pr: &GhPullRequest,
        pr_status: &PullRequestStatus,
    ) -> Result<TryMergePullRequestState>;
}

#[derive(Component)]
#[shaku(interface = AutomergePullRequestInterface)]
pub(crate) struct AutomergePullRequest;

#[async_trait]
impl AutomergePullRequestInterface for AutomergePullRequest {
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        pr_handle: &PullRequestHandle,
        upstream_pr: &GhPullRequest,
        pr_status: &PullRequestStatus,
    ) -> Result<TryMergePullRequestState> {
        let try_merge_uc: &dyn TryMergePullRequestFromStatusInterface =
            ctx.core_module.resolve_ref();
        let result = try_merge_uc
            .run(ctx, pr_handle, upstream_pr, pr_status, None)
            .await?;

        match result {
            TryMergePullRequestState::Success(strategy) => {
                CommentApi::post_comment(
                    ctx.config,
                    ctx.api_service,
                    pr_handle.repository_path().owner(),
                    pr_handle.repository_path().name(),
                    pr_handle.number(),
                    &format!(
                        "Pull request successfully auto-merged! (strategy: '{}')",
                        strategy
                    ),
                )
                .await?;

                Ok(result)
            }
            TryMergePullRequestState::Error => {
                CommentApi::post_comment(
                    ctx.config,
                    ctx.api_service,
                    pr_handle.repository_path().owner(),
                    pr_handle.repository_path().name(),
                    pr_handle.number(),
                    "Could not auto-merge this pull request because of an error.\nAuto-merge disabled.",
                )
                .await?;

                Ok(result)
            }
            other => Ok(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use prbot_database_interface::DbService;
    use prbot_ghapi_interface::MockApiService;
    use prbot_models::{MergeStrategy, PullRequest, Repository};

    use super::*;
    use crate::{
        context::tests::CoreContextTest,
        use_cases::pulls::MockTryMergePullRequestFromStatusInterface, CoreModule,
    };

    struct Arrange {
        ctx: CoreContextTest,
        repository: Repository,
        pull_request: PullRequest,
        upstream_pr: GhPullRequest,
        pr_status: PullRequestStatus,
    }

    impl Arrange {
        async fn check(&self, state: TryMergePullRequestState) {
            let result = AutomergePullRequest
                .run(
                    &self.ctx.as_context(),
                    &PullRequestHandle::new(self.repository.path(), self.pull_request.number),
                    &self.upstream_pr,
                    &self.pr_status,
                )
                .await
                .unwrap();

            assert_eq!(result, state);
        }

        fn arrange_try_merge_pull_request_response(&mut self, state: TryMergePullRequestState) {
            let mut mock = MockTryMergePullRequestFromStatusInterface::new();
            mock.expect_run().return_once(|_, _, _, _, _| Ok(state));

            self.ctx.core_module = CoreModule::builder()
                .with_component_override::<dyn TryMergePullRequestFromStatusInterface>(Box::new(
                    mock,
                ))
                .build();
        }
    }

    async fn arrange() -> Arrange {
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

        let pr = ctx
            .db_service
            .pull_requests_create(PullRequest {
                repository_id: repo.id,
                number: 1,
                ..Default::default()
            })
            .await
            .unwrap();

        let upstream_pr = GhPullRequest {
            ..Default::default()
        };

        Arrange {
            ctx,
            repository: repo,
            pull_request: pr,
            upstream_pr,
            pr_status: PullRequestStatus {
                ..Default::default()
            },
        }
    }

    #[tokio::test]
    async fn run_success() {
        let mut arrange = arrange().await;
        arrange.arrange_try_merge_pull_request_response(TryMergePullRequestState::Success(
            MergeStrategy::Merge,
        ));

        arrange.ctx.api_service = {
            let mut svc = MockApiService::new();
            svc.expect_comments_post()
                .once()
                .withf(|owner, name, number, body| {
                    owner == "owner"
                        && name == "name"
                        && number == &1
                        && body.contains("successfully")
                })
                .return_once(|_, _, _, _| Ok(1));

            svc
        };

        arrange
            .check(TryMergePullRequestState::Success(MergeStrategy::Merge))
            .await;
    }

    #[tokio::test]
    async fn run_error() {
        let mut arrange = arrange().await;
        arrange.arrange_try_merge_pull_request_response(TryMergePullRequestState::Error);

        arrange.ctx.api_service = {
            let mut svc = MockApiService::new();
            svc.expect_comments_post()
                .once()
                .withf(|owner, name, number, body| {
                    owner == "owner" && name == "name" && number == &1 && body.contains("an error")
                })
                .return_once(|_, _, _, _| Ok(1));

            svc
        };

        arrange.check(TryMergePullRequestState::Error).await;
    }

    #[tokio::test]
    async fn run_not_ready() {
        let mut arrange = arrange().await;
        arrange.arrange_try_merge_pull_request_response(TryMergePullRequestState::NotReady);

        arrange.check(TryMergePullRequestState::NotReady).await;
    }

    #[tokio::test]
    async fn run_already_locked() {
        let mut arrange = arrange().await;
        arrange.arrange_try_merge_pull_request_response(TryMergePullRequestState::AlreadyLocked);

        arrange.check(TryMergePullRequestState::AlreadyLocked).await;
    }
}
