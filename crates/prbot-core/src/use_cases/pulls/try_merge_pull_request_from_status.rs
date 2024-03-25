use async_trait::async_trait;
use prbot_ghapi_interface::types::GhPullRequest;
use prbot_lock_interface::{using_lock, UsingLockResult};
use prbot_models::{MergeStrategy, PullRequestHandle, StepLabel};
use shaku::{Component, HasComponent, Interface};
use tracing::error;

use super::MergePullRequestInterface;
use crate::{
    use_cases::status::{PullRequestStatus, StepLabelChooser},
    CoreContext, DomainError, Result,
};

#[derive(Debug, PartialEq, Clone)]
pub enum TryMergePullRequestState {
    AlreadyLocked,
    NotReady,
    Error,
    Success(MergeStrategy),
}

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait]
pub trait TryMergePullRequestFromStatusInterface: Interface {
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        pr_handle: &PullRequestHandle,
        upstream_pr: &GhPullRequest,
        pr_status: &PullRequestStatus,
        strategy: Option<MergeStrategy>,
    ) -> Result<TryMergePullRequestState>;
}

#[derive(Component)]
#[shaku(interface = TryMergePullRequestFromStatusInterface)]
pub(crate) struct TryMergePullRequestFromStatus;

#[async_trait]
impl TryMergePullRequestFromStatusInterface for TryMergePullRequestFromStatus {
    #[tracing::instrument(skip(self, ctx), fields(pr_handle, merge_strategy))]
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        pr_handle: &PullRequestHandle,
        upstream_pr: &GhPullRequest,
        pr_status: &PullRequestStatus,
        strategy: Option<MergeStrategy>,
    ) -> Result<TryMergePullRequestState> {
        let step = StepLabelChooser::default().choose_from_status(pr_status);

        // Use lock
        let key = format!(
            "pr-merge-{}-{}-{}",
            pr_handle.repository_path().owner(),
            pr_handle.repository_path().name(),
            pr_handle.number()
        );

        // Use step to determine merge possibility
        if step == StepLabel::AwaitingMerge && upstream_pr.merged != Some(true) {
            let output =
                using_lock::<_, _, _, DomainError>(ctx.lock_service, &key, 1_000, || async {
                    let strategy = strategy.unwrap_or(pr_status.merge_strategy);
                    let uc: &dyn MergePullRequestInterface = ctx.core_module.resolve_ref();

                    let merge_result = uc.run(ctx, pr_handle, strategy, upstream_pr).await;
                    match merge_result {
                        Ok(()) => Ok(TryMergePullRequestState::Success(strategy)),
                        Err(e) => {
                            error!(
                                owner = %pr_handle.owner(),
                                name = %pr_handle.name(),
                                pr_number = pr_handle.number(),
                                error = %e,
                                message = "Error while merging pull request"
                            );

                            Ok(TryMergePullRequestState::Error)
                        }
                    }
                })
                .await?;

            match output {
                UsingLockResult::AlreadyLocked => Ok(TryMergePullRequestState::AlreadyLocked),
                UsingLockResult::Locked(data) => Ok(data?),
            }
        } else {
            Ok(TryMergePullRequestState::NotReady)
        }
    }
}

#[cfg(test)]
mod tests {
    use prbot_database_interface::DbService;
    use prbot_ghapi_interface::{types::GhPullRequest, ApiError};
    use prbot_lock_interface::{LockInstance, LockStatus, MockLockService};
    use prbot_models::{
        ChecksStatus, MergeStrategy, PullRequest, PullRequestHandle, QaStatus, Repository,
    };

    use crate::{
        context::tests::CoreContextTest,
        use_cases::{
            pulls::{
                try_merge_pull_request_from_status::{
                    TryMergePullRequestFromStatus, TryMergePullRequestState,
                },
                TryMergePullRequestFromStatusInterface,
            },
            status::PullRequestStatus,
        },
    };

    struct Arrange {
        ctx: CoreContextTest,
        repository: Repository,
        pull_request: PullRequest,
        pr_status: PullRequestStatus,
        upstream_pr: GhPullRequest,
    }

    impl Arrange {
        async fn check(&self, state: TryMergePullRequestState) {
            let result = TryMergePullRequestFromStatus
                .run(
                    &self.ctx.as_context(),
                    &PullRequestHandle::new(self.repository.path(), self.pull_request.number),
                    &self.upstream_pr,
                    &self.pr_status,
                    None,
                )
                .await
                .unwrap();

            assert_eq!(result, state);
        }
    }

    async fn arrange(pr_status: PullRequestStatus) -> Arrange {
        let mut ctx = CoreContextTest::new();
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

        ctx.lock_service
            .expect_wait_lock_resource()
            .withf(|name, timeout| name == "pr-merge-owner-name-1" && *timeout == 1_000)
            .return_once(|name, _| {
                Ok(LockStatus::SuccessfullyLocked(LockInstance::new_dummy(
                    name,
                )))
            });

        Arrange {
            ctx,
            repository: repo,
            pull_request: pr,
            pr_status,
            upstream_pr,
        }
    }

    #[tokio::test]
    async fn not_ready() {
        let arrange = arrange(PullRequestStatus {
            checks_status: ChecksStatus::Pass,
            qa_status: QaStatus::Pass,
            // Not ready!
            valid_pr_title: false,
            wip: false,
            mergeable: true,
            merged: false,
            ..Default::default()
        })
        .await;

        arrange.check(TryMergePullRequestState::NotReady).await;
    }

    #[tokio::test]
    async fn error() {
        let mut arrange = arrange(PullRequestStatus {
            checks_status: ChecksStatus::Pass,
            qa_status: QaStatus::Pass,
            valid_pr_title: true,
            wip: false,
            mergeable: true,
            merged: false,
            ..Default::default()
        })
        .await;

        arrange
            .ctx
            .api_service
            .expect_pulls_merge()
            .once()
            .return_once(|_, _, _, _, _, _| {
                Err(ApiError::MergeError {
                    pr_number: 1,
                    repository_path: "me/test".into(),
                })
            });

        arrange.check(TryMergePullRequestState::Error).await;
    }

    #[tokio::test]
    async fn success() {
        let mut arrange = arrange(PullRequestStatus {
            checks_status: ChecksStatus::Pass,
            qa_status: QaStatus::Pass,
            valid_pr_title: true,
            wip: false,
            mergeable: true,
            merged: false,
            ..Default::default()
        })
        .await;

        arrange
            .ctx
            .api_service
            .expect_pulls_merge()
            .once()
            .return_once(|_, _, _, _, _, _| Ok(()));

        arrange
            .check(TryMergePullRequestState::Success(MergeStrategy::Merge))
            .await;
    }

    #[tokio::test]
    async fn already_locked() {
        let mut arrange = arrange(PullRequestStatus {
            checks_status: ChecksStatus::Pass,
            qa_status: QaStatus::Pass,
            valid_pr_title: true,
            wip: false,
            mergeable: true,
            merged: false,
            ..Default::default()
        })
        .await;

        let mut lock_service = MockLockService::new();
        lock_service
            .expect_wait_lock_resource()
            .withf(|name, timeout| name == "pr-merge-owner-name-1" && *timeout == 1_000)
            .return_once(|_, _| Ok(LockStatus::AlreadyLocked));

        arrange.ctx.lock_service = lock_service;

        arrange.check(TryMergePullRequestState::AlreadyLocked).await;
    }
}
