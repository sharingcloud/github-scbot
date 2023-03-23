use async_trait::async_trait;
use github_scbot_database_interface::DbService;
use github_scbot_domain_models::{PullRequestHandle, StepLabel};
use github_scbot_ghapi_interface::{types::GhPullRequest, ApiService};
use github_scbot_lock_interface::{LockService, LockStatus};

use super::{
    build_pull_request_status::BuildPullRequestStatusUseCaseInterface,
    utils::StatusMessageGenerator, StepLabelChooser,
};
use crate::{
    use_cases::{
        pulls::{AutomergePullRequestUseCaseInterface, SetStepLabelUseCaseInterface},
        summary::PostSummaryCommentUseCaseInterface,
    },
    Result,
};

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait(?Send)]
pub trait UpdatePullRequestStatusUseCaseInterface {
    async fn run(&self, pr_handle: &PullRequestHandle, upstream_pr: &GhPullRequest) -> Result<()>;
}

pub struct UpdatePullRequestStatusUseCase<'a> {
    pub api_service: &'a dyn ApiService,
    pub db_service: &'a dyn DbService,
    pub lock_service: &'a dyn LockService,
    pub build_pull_request_status: &'a dyn BuildPullRequestStatusUseCaseInterface,
    pub set_step_label: &'a dyn SetStepLabelUseCaseInterface,
    pub automerge_pull_request: &'a dyn AutomergePullRequestUseCaseInterface,
    pub post_summary_comment: &'a dyn PostSummaryCommentUseCaseInterface,
}

#[async_trait(?Send)]
impl<'a> UpdatePullRequestStatusUseCaseInterface for UpdatePullRequestStatusUseCase<'a> {
    #[tracing::instrument(
        skip_all,
        fields(
            pr_handle,
            head_sha = %upstream_pr.head.sha
        )
    )]
    async fn run(&self, pr_handle: &PullRequestHandle, upstream_pr: &GhPullRequest) -> Result<()> {
        let commit_sha = &upstream_pr.head.sha;
        let pr_status = self
            .build_pull_request_status
            .run(pr_handle, upstream_pr)
            .await?;

        // Update step label.
        let step_label = StepLabelChooser::default().choose_from_status(&pr_status);
        self.set_step_label.run(pr_handle, Some(step_label)).await?;

        // Post status.
        self.post_summary_comment.run(pr_handle, &pr_status).await?;

        // Create or update status.
        let status_message = StatusMessageGenerator::default().generate(&pr_status)?;

        self.api_service
            .commit_statuses_update(
                pr_handle.repository().owner(),
                pr_handle.repository().name(),
                commit_sha,
                status_message.state,
                status_message.title,
                &status_message.message,
            )
            .await?;

        let pr_model = self
            .db_service
            .pull_requests_get(
                pr_handle.repository().owner(),
                pr_handle.repository().name(),
                pr_handle.number(),
            )
            .await?
            .unwrap();

        // Merge if auto-merge is enabled
        if matches!(step_label, StepLabel::AwaitingMerge)
            && upstream_pr.merged != Some(true)
            && pr_model.automerge
        {
            // Use lock
            let key = format!(
                "pr-merge_{}-{}_{}",
                pr_handle.repository().owner(),
                pr_handle.repository().name(),
                pr_handle.number()
            );
            if let LockStatus::SuccessfullyLocked(l) =
                self.lock_service.try_lock_resource(&key).await?
            {
                if !self
                    .automerge_pull_request
                    .run(pr_handle, upstream_pr)
                    .await?
                {
                    self.db_service
                        .pull_requests_set_automerge(
                            pr_handle.repository().owner(),
                            pr_handle.repository().name(),
                            pr_handle.number(),
                            false,
                        )
                        .await?;

                    // Update status
                    self.post_summary_comment.run(pr_handle, &pr_status).await?;
                }

                l.release().await?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use build_pull_request_status::MockBuildPullRequestStatusUseCaseInterface;
    use github_scbot_database_memory::MemoryDb;
    use github_scbot_domain_models::{ChecksStatus, PullRequest, QaStatus, Repository};
    use github_scbot_ghapi_interface::{
        types::{GhBranch, GhCommitStatus},
        MockApiService,
    };
    use github_scbot_lock_interface::{LockInstance, MockLockService};

    use super::*;
    use crate::use_cases::{
        pulls::{MockAutomergePullRequestUseCaseInterface, MockSetStepLabelUseCaseInterface},
        status::{build_pull_request_status, PullRequestStatus},
        summary::MockPostSummaryCommentUseCaseInterface,
    };

    #[tokio::test]
    async fn update() {
        let automerge_pull_request = MockAutomergePullRequestUseCaseInterface::new();
        let lock_service = MockLockService::new();

        let api_service = {
            let mut svc = MockApiService::new();

            svc.expect_commit_statuses_update()
                .once()
                .withf(|owner, name, sha, status, title, body| {
                    owner == "me"
                        && name == "test"
                        && sha == "abcdef"
                        && *status == GhCommitStatus::Pending
                        && title == "Validation"
                        && body == "Waiting on reviews"
                })
                .return_once(|_, _, _, _, _, _| Ok(()));

            svc
        };

        let db_service = {
            let svc = MemoryDb::new();

            let repo = svc
                .repositories_create(Repository {
                    owner: "me".into(),
                    name: "test".into(),
                    default_enable_checks: false,
                    default_enable_qa: false,
                    default_needed_reviewers_count: 1,
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

        let set_step_label = {
            let mut mock = MockSetStepLabelUseCaseInterface::new();
            mock.expect_run()
                .once()
                .withf(|handle, label| {
                    handle == &("me", "test", 1).into() && *label == Some(StepLabel::AwaitingReview)
                })
                .return_once(|_, _| Ok(()));

            mock
        };

        let post_summary_comment = {
            let mut mock = MockPostSummaryCommentUseCaseInterface::new();

            mock.expect_run()
                .once()
                .withf(|pr_handle, _status| pr_handle == &("me", "test", 1).into())
                .return_once(|_, _| Ok(()));

            mock
        };

        let build_pull_request_status = {
            let mut mock = MockBuildPullRequestStatusUseCaseInterface::new();

            mock.expect_run()
                .once()
                .withf(|pr_handle, upstream_pr| {
                    pr_handle == &("me", "test", 1).into() && upstream_pr.number == 1
                })
                .return_once(|_, _| {
                    Ok(PullRequestStatus {
                        checks_status: ChecksStatus::Pass,
                        valid_pr_title: true,
                        wip: false,
                        mergeable: true,
                        merged: false,
                        needed_reviewers_count: 1,
                        ..Default::default()
                    })
                });

            mock
        };

        UpdatePullRequestStatusUseCase {
            api_service: &api_service,
            db_service: &db_service,
            lock_service: &lock_service,
            set_step_label: &set_step_label,
            automerge_pull_request: &automerge_pull_request,
            post_summary_comment: &post_summary_comment,
            build_pull_request_status: &build_pull_request_status,
        }
        .run(
            &("me", "test", 1).into(),
            &GhPullRequest {
                number: 1,
                head: GhBranch {
                    sha: "abcdef".into(),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn automerge_success() {
        let api_service = {
            let mut svc = MockApiService::new();

            svc.expect_commit_statuses_update()
                .once()
                .withf(|owner, name, sha, status, title, body| {
                    owner == "me"
                        && name == "test"
                        && sha == "abcdef"
                        && *status == GhCommitStatus::Success
                        && title == "Validation"
                        && body == "All good."
                })
                .return_once(|_, _, _, _, _, _| Ok(()));

            svc
        };

        let db_service = {
            let svc = MemoryDb::new();

            let repo = svc
                .repositories_create(Repository {
                    owner: "me".into(),
                    name: "test".into(),
                    default_enable_checks: false,
                    default_enable_qa: false,
                    default_needed_reviewers_count: 0,
                    default_automerge: true,
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

        let lock_service = {
            let mut svc = MockLockService::new();

            svc.expect_try_lock_resource()
                .once()
                .withf(|name| name == "pr-merge_me-test_1")
                .return_once(|_| {
                    Ok(LockStatus::SuccessfullyLocked(LockInstance::new_dummy(
                        "dummy",
                    )))
                });

            svc
        };

        let automerge_pull_request = {
            let mut automerge_pull_request = MockAutomergePullRequestUseCaseInterface::new();
            automerge_pull_request
                .expect_run()
                .once()
                .withf(|handle, upstream| {
                    handle == &("me", "test", 1).into() && upstream.number == 1
                })
                .return_once(|_, _| Ok(true));
            automerge_pull_request
        };

        let set_step_label = {
            let mut set_step_label = MockSetStepLabelUseCaseInterface::new();
            set_step_label
                .expect_run()
                .once()
                .withf(|handle, label| {
                    handle == &("me", "test", 1).into() && *label == Some(StepLabel::AwaitingMerge)
                })
                .return_once(|_, _| Ok(()));

            set_step_label
        };

        let post_summary_comment = {
            let mut mock = MockPostSummaryCommentUseCaseInterface::new();

            mock.expect_run()
                .once()
                .withf(|pr_handle, _status| pr_handle == &("me", "test", 1).into())
                .return_once(|_, _| Ok(()));

            mock
        };

        let build_pull_request_status = {
            let mut mock = MockBuildPullRequestStatusUseCaseInterface::new();

            mock.expect_run()
                .once()
                .withf(|pr_handle, upstream_pr| {
                    pr_handle == &("me", "test", 1).into() && upstream_pr.number == 1
                })
                .return_once(|_, _| {
                    Ok(PullRequestStatus {
                        checks_status: ChecksStatus::Pass,
                        qa_status: QaStatus::Pass,
                        valid_pr_title: true,
                        wip: false,
                        mergeable: true,
                        merged: false,
                        needed_reviewers_count: 0,
                        ..Default::default()
                    })
                });

            mock
        };

        UpdatePullRequestStatusUseCase {
            api_service: &api_service,
            db_service: &db_service,
            lock_service: &lock_service,
            set_step_label: &set_step_label,
            automerge_pull_request: &automerge_pull_request,
            post_summary_comment: &post_summary_comment,
            build_pull_request_status: &build_pull_request_status,
        }
        .run(
            &("me", "test", 1).into(),
            &GhPullRequest {
                number: 1,
                title: "Sample".into(),
                head: GhBranch {
                    sha: "abcdef".into(),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn automerge_failure() {
        let api_service = {
            let mut svc = MockApiService::new();

            svc.expect_commit_statuses_update()
                .once()
                .withf(|owner, name, sha, status, title, body| {
                    owner == "me"
                        && name == "test"
                        && sha == "abcdef"
                        && *status == GhCommitStatus::Success
                        && title == "Validation"
                        && body == "All good."
                })
                .return_once(|_, _, _, _, _, _| Ok(()));

            svc
        };

        let db_service = {
            let svc = MemoryDb::new();

            let repo = svc
                .repositories_create(Repository {
                    owner: "me".into(),
                    name: "test".into(),
                    default_enable_checks: false,
                    default_enable_qa: false,
                    default_needed_reviewers_count: 0,
                    default_automerge: true,
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

        let lock_service = {
            let mut svc = MockLockService::new();

            svc.expect_try_lock_resource()
                .once()
                .withf(|name| name == "pr-merge_me-test_1")
                .return_once(|_| {
                    Ok(LockStatus::SuccessfullyLocked(LockInstance::new_dummy(
                        "dummy",
                    )))
                });

            svc
        };

        let set_step_label = {
            let mut mock = MockSetStepLabelUseCaseInterface::new();

            mock.expect_run()
                .once()
                .withf(|handle, label| {
                    handle == &("me", "test", 1).into() && *label == Some(StepLabel::AwaitingMerge)
                })
                .return_once(|_, _| Ok(()));

            mock
        };

        let automerge_pull_request = {
            let mut mock = MockAutomergePullRequestUseCaseInterface::new();

            mock.expect_run()
                .once()
                .withf(|handle, upstream| {
                    handle == &("me", "test", 1).into() && upstream.number == 1
                })
                .return_once(|_, _| Ok(false));

            mock
        };

        let post_summary_comment = {
            let mut mock = MockPostSummaryCommentUseCaseInterface::new();

            mock.expect_run()
                .once()
                .withf(|pr_handle, _status| pr_handle == &("me", "test", 1).into())
                .return_once(|_, _| Ok(()));

            mock.expect_run()
                .once()
                .withf(|pr_handle, _status| pr_handle == &("me", "test", 1).into())
                .return_once(|_, _| Ok(()));

            mock
        };

        let build_pull_request_status = {
            let mut mock = MockBuildPullRequestStatusUseCaseInterface::new();

            mock.expect_run()
                .once()
                .withf(|pr_handle, upstream_pr| {
                    pr_handle == &("me", "test", 1).into() && upstream_pr.number == 1
                })
                .return_once(|_, _| {
                    Ok(PullRequestStatus {
                        checks_status: ChecksStatus::Pass,
                        qa_status: QaStatus::Pass,
                        valid_pr_title: true,
                        wip: false,
                        mergeable: true,
                        merged: false,
                        needed_reviewers_count: 0,
                        ..Default::default()
                    })
                });

            mock
        };

        UpdatePullRequestStatusUseCase {
            api_service: &api_service,
            db_service: &db_service,
            lock_service: &lock_service,
            set_step_label: &set_step_label,
            automerge_pull_request: &automerge_pull_request,
            post_summary_comment: &post_summary_comment,
            build_pull_request_status: &build_pull_request_status,
        }
        .run(
            &("me", "test", 1).into(),
            &GhPullRequest {
                number: 1,
                title: "Sample".into(),
                head: GhBranch {
                    sha: "abcdef".into(),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .await
        .unwrap();

        // Merge error resets automerge status.
        assert!(
            !db_service
                .pull_requests_get("me", "test", 1)
                .await
                .unwrap()
                .unwrap()
                .automerge
        );
    }
}
