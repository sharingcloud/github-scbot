use github_scbot_database_interface::DbService;
use github_scbot_domain_models::{PullRequestHandle, StepLabel};
use github_scbot_ghapi_interface::{comments::CommentApi, types::GhPullRequest, ApiService};
use github_scbot_lock_interface::{LockService, LockStatus};

use super::{
    build_pull_request_status::BuildPullRequestStatusUseCase, utils::StatusMessageGenerator,
    StepLabelChooser,
};
use crate::{
    use_cases::{
        pulls::{
            DeterminePullRequestMergeStrategyUseCase, MergePullRequestUseCase, SetStepLabelUseCase,
        },
        summary::PostSummaryCommentUseCase,
    },
    Result,
};

pub struct UpdatePullRequestStatusUseCase<'a> {
    pub api_service: &'a dyn ApiService,
    pub db_service: &'a dyn DbService,
    pub lock_service: &'a dyn LockService,
}

impl<'a> UpdatePullRequestStatusUseCase<'a> {
    #[tracing::instrument(
        skip_all,
        fields(
            pr_handle,
            head_sha = %upstream_pr.head.sha
        )
    )]
    pub async fn run(
        &self,
        pr_handle: &PullRequestHandle,
        upstream_pr: &GhPullRequest,
    ) -> Result<()> {
        let commit_sha = &upstream_pr.head.sha;
        let pr_status = BuildPullRequestStatusUseCase {
            api_service: self.api_service,
            db_service: self.db_service,
        }
        .run(pr_handle, upstream_pr)
        .await?;

        // Update step label.
        let step_label = StepLabelChooser::default().choose_from_status(&pr_status);

        self.apply_pull_request_step(pr_handle, Some(step_label))
            .await?;

        // Post status.
        PostSummaryCommentUseCase {
            api_service: self.api_service,
            db_service: self.db_service,
            lock_service: self.lock_service,
        }
        .run(pr_handle, &pr_status)
        .await?;

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
                    .try_automerge_pull_request(pr_handle, upstream_pr)
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
                    PostSummaryCommentUseCase {
                        api_service: self.api_service,
                        db_service: self.db_service,
                        lock_service: self.lock_service,
                    }
                    .run(pr_handle, &pr_status)
                    .await?;
                }

                l.release().await?;
            }
        }

        Ok(())
    }

    /// Apply pull request step.
    #[tracing::instrument(skip(self))]
    async fn apply_pull_request_step(
        &self,
        pr_handle: &PullRequestHandle,
        step: Option<StepLabel>,
    ) -> Result<()> {
        SetStepLabelUseCase {
            api_service: self.api_service,
        }
        .run(pr_handle, step)
        .await
        .map_err(Into::into)
    }

    #[tracing::instrument(skip_all, fields(pr_handle), ret)]
    async fn try_automerge_pull_request(
        &self,
        pr_handle: &PullRequestHandle,
        upstream_pr: &GhPullRequest,
    ) -> Result<bool> {
        let repository = self
            .db_service
            .repositories_get(
                pr_handle.repository().owner(),
                pr_handle.repository().name(),
            )
            .await?
            .unwrap();
        let pull_request = self
            .db_service
            .pull_requests_get(
                pr_handle.repository().owner(),
                pr_handle.repository().name(),
                pr_handle.number(),
            )
            .await?
            .unwrap();

        let strategy = if let Some(s) = pull_request.strategy_override {
            s
        } else {
            DeterminePullRequestMergeStrategyUseCase {
                db_service: self.db_service,
            }
            .run(
                pr_handle.repository(),
                &upstream_pr.base.reference,
                &upstream_pr.head.reference,
                repository.default_strategy,
            )
            .await?
        };

        let merge_result = MergePullRequestUseCase {
            api_service: self.api_service,
        }
        .run(pr_handle, strategy, upstream_pr)
        .await;

        match merge_result {
            Ok(()) => {
                CommentApi::post_comment(
                    self.api_service,
                    pr_handle.repository().owner(),
                    pr_handle.repository().name(),
                    pr_handle.number(),
                    &format!(
                        "Pull request successfully auto-merged! (strategy: '{}')",
                        strategy
                    ),
                )
                .await?;
                Ok(true)
            }
            Err(e) => {
                CommentApi::post_comment(
                    self.api_service,
                    pr_handle.repository().owner(),
                    pr_handle.repository().name(),
                    pr_handle.number(),
                    &format!(
                        "Could not auto-merge this pull request: _{}_\nAuto-merge disabled",
                        e
                    ),
                )
                .await?;
                Ok(false)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_database_memory::MemoryDb;
    use github_scbot_domain_models::{PullRequest, Repository};
    use github_scbot_ghapi_interface::{
        types::{GhBranch, GhCommitStatus, GhMergeStrategy},
        ApiError, MockApiService,
    };
    use github_scbot_lock_interface::{LockInstance, MockLockService};

    use super::*;

    #[tokio::test]
    async fn update() {
        let mut api_service = MockApiService::new();
        let db_service = MemoryDb::new();
        let mut lock_service = MockLockService::new();

        let repo = db_service
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

        let _ = db_service
            .pull_requests_create(
                PullRequest {
                    number: 1,
                    ..Default::default()
                }
                .with_repository(&repo),
            )
            .await
            .unwrap();

        let upstream_pr = GhPullRequest {
            number: 1,
            head: GhBranch {
                sha: "abcdef".into(),
                ..Default::default()
            },
            ..Default::default()
        };

        lock_service
            .expect_wait_lock_resource()
            .once()
            .withf(|name, timeout| name == "summary-me-test-1" && timeout == &10000)
            .return_once(|_, _| {
                Ok(LockStatus::SuccessfullyLocked(LockInstance::new_dummy(
                    "dummy",
                )))
            });

        api_service
            .expect_pull_reviews_list()
            .once()
            .withf(|owner, name, number| owner == "me" && name == "test" && number == &1)
            .return_once(|_, _, _| Ok(vec![]));

        api_service
            .expect_issue_labels_list()
            .once()
            .withf(|owner, name, issue_id| owner == "me" && name == "test" && issue_id == &1)
            .return_once(|_, _, _| Ok(vec![]));

        api_service
            .expect_issue_labels_replace_all()
            .once()
            .withf(|owner, name, issue_id, labels| {
                owner == "me"
                    && name == "test"
                    && issue_id == &1
                    && labels == ["step/awaiting-review".to_string()]
            })
            .return_once(|_, _, _, _| Ok(()));

        api_service
            .expect_comments_post()
            .once()
            .withf(|owner, name, pr_id, text| {
                owner == "me" && name == "test" && pr_id == &1 && !text.is_empty()
            })
            .return_once(|_, _, _, _| Ok(1));

        api_service
            .expect_commit_statuses_update()
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

        UpdatePullRequestStatusUseCase {
            api_service: &api_service,
            db_service: &db_service,
            lock_service: &lock_service,
        }
        .run(&("me", "test", 1).into(), &upstream_pr)
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn automerge_success() {
        let mut api_service = MockApiService::new();
        let db_service = MemoryDb::new();
        let mut lock_service = MockLockService::new();

        let repo = db_service
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

        let _ = db_service
            .pull_requests_create(
                PullRequest {
                    number: 1,
                    ..Default::default()
                }
                .with_repository(&repo),
            )
            .await
            .unwrap();

        let upstream_pr = GhPullRequest {
            number: 1,
            title: "Sample".into(),
            head: GhBranch {
                sha: "abcdef".into(),
                ..Default::default()
            },
            ..Default::default()
        };

        lock_service
            .expect_wait_lock_resource()
            .once()
            .withf(|name, timeout| name == "summary-me-test-1" && timeout == &10000)
            .return_once(|_, _| {
                Ok(LockStatus::SuccessfullyLocked(LockInstance::new_dummy(
                    "dummy",
                )))
            });

        lock_service
            .expect_try_lock_resource()
            .once()
            .withf(|name| name == "pr-merge_me-test_1")
            .return_once(|_| {
                Ok(LockStatus::SuccessfullyLocked(LockInstance::new_dummy(
                    "dummy",
                )))
            });

        api_service
            .expect_pull_reviews_list()
            .once()
            .withf(|owner, name, number| owner == "me" && name == "test" && number == &1)
            .return_once(|_, _, _| Ok(vec![]));

        api_service
            .expect_issue_labels_list()
            .once()
            .withf(|owner, name, issue_id| owner == "me" && name == "test" && issue_id == &1)
            .return_once(|_, _, _| Ok(vec![]));

        api_service
            .expect_issue_labels_replace_all()
            .once()
            .withf(|owner, name, issue_id, labels| {
                owner == "me"
                    && name == "test"
                    && issue_id == &1
                    && labels == ["step/awaiting-merge".to_string()]
            })
            .return_once(|_, _, _, _| Ok(()));

        api_service
            .expect_comments_post()
            .once()
            .withf(|owner, name, pr_id, text| {
                owner == "me" && name == "test" && pr_id == &1 && !text.is_empty()
            })
            .return_once(|_, _, _, _| Ok(1));

        api_service
            .expect_comments_post()
            .once()
            .withf(|owner, name, pr_id, text| {
                owner == "me"
                    && name == "test"
                    && pr_id == &1
                    && text
                        .starts_with("Pull request successfully auto-merged! (strategy: 'merge')")
            })
            .return_once(|_, _, _, _| Ok(1));

        api_service
            .expect_commit_statuses_update()
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

        api_service
            .expect_pulls_merge()
            .once()
            .withf(|owner, name, issue_number, title, message, strategy| {
                owner == "me"
                    && name == "test"
                    && issue_number == &1
                    && title == "Sample (#1)"
                    && message.is_empty()
                    && *strategy == GhMergeStrategy::Merge
            })
            .return_once(|_, _, _, _, _, _| Ok(()));

        UpdatePullRequestStatusUseCase {
            api_service: &api_service,
            db_service: &db_service,
            lock_service: &lock_service,
        }
        .run(&("me", "test", 1).into(), &upstream_pr)
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn automerge_failure() {
        let mut api_service = MockApiService::new();
        let db_service = MemoryDb::new();
        let mut lock_service = MockLockService::new();

        let repo = db_service
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

        let _ = db_service
            .pull_requests_create(
                PullRequest {
                    number: 1,
                    ..Default::default()
                }
                .with_repository(&repo),
            )
            .await
            .unwrap();

        let upstream_pr = GhPullRequest {
            number: 1,
            title: "Sample".into(),
            head: GhBranch {
                sha: "abcdef".into(),
                ..Default::default()
            },
            ..Default::default()
        };

        lock_service
            .expect_wait_lock_resource()
            .once()
            .withf(|name, timeout| name == "summary-me-test-1" && timeout == &10000)
            .return_once(|_, _| {
                Ok(LockStatus::SuccessfullyLocked(LockInstance::new_dummy(
                    "dummy",
                )))
            });

        lock_service
            .expect_try_lock_resource()
            .once()
            .withf(|name| name == "pr-merge_me-test_1")
            .return_once(|_| {
                Ok(LockStatus::SuccessfullyLocked(LockInstance::new_dummy(
                    "dummy",
                )))
            });

        api_service
            .expect_pull_reviews_list()
            .once()
            .withf(|owner, name, number| owner == "me" && name == "test" && number == &1)
            .return_once(|_, _, _| Ok(vec![]));

        api_service
            .expect_issue_labels_list()
            .once()
            .withf(|owner, name, issue_id| owner == "me" && name == "test" && issue_id == &1)
            .return_once(|_, _, _| Ok(vec![]));

        api_service
            .expect_issue_labels_replace_all()
            .once()
            .withf(|owner, name, issue_id, labels| {
                owner == "me"
                    && name == "test"
                    && issue_id == &1
                    && labels == ["step/awaiting-merge".to_string()]
            })
            .return_once(|_, _, _, _| Ok(()));

        api_service
            .expect_comments_post()
            .once()
            .withf(|owner, name, pr_id, text| {
                owner == "me" && name == "test" && pr_id == &1 && !text.is_empty()
            })
            .return_once(|_, _, _, _| Ok(1));

        api_service
            .expect_comments_post()
            .once()
            .withf(|owner, name, pr_id, text| {
                owner == "me"
                    && name == "test"
                    && pr_id == &1
                    && text.starts_with("Could not auto-merge this pull request")
            })
            .return_once(|_, _, _, _| Ok(2));

        api_service
            .expect_comments_update()
            .once()
            .withf(|owner, name, comment_id, body| {
                owner == "me" && name == "test" && comment_id == &1 && !body.is_empty()
            })
            .return_once(|_, _, _, _| Ok(1));

        api_service
            .expect_commit_statuses_update()
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

        api_service
            .expect_pulls_merge()
            .once()
            .withf(|owner, name, issue_number, title, message, strategy| {
                owner == "me"
                    && name == "test"
                    && issue_number == &1
                    && title == "Sample (#1)"
                    && message.is_empty()
                    && *strategy == GhMergeStrategy::Merge
            })
            .return_once(|_, _, _, _, _, _| {
                Err(ApiError::MergeError {
                    pr_number: 1,
                    repository_path: "me/test".into(),
                })
            });

        UpdatePullRequestStatusUseCase {
            api_service: &api_service,
            db_service: &db_service,
            lock_service: &lock_service,
        }
        .run(&("me", "test", 1).into(), &upstream_pr)
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
