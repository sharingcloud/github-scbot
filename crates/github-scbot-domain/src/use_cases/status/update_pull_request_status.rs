use github_scbot_database_interface::DbService;
use github_scbot_domain_models::StepLabel;
use github_scbot_ghapi_interface::{comments::CommentApi, types::GhPullRequest, ApiService};
use github_scbot_lock_interface::{LockService, LockStatus};

use super::{
    build_pull_request_status::BuildPullRequestStatusUseCase,
    determine_automatic_step::DetermineAutomaticStepUseCase,
    generate_status_message::GenerateStatusMessageUseCase,
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
    pub repo_owner: &'a str,
    pub repo_name: &'a str,
    pub pr_number: u64,
    pub upstream_pr: &'a GhPullRequest,
}

impl<'a> UpdatePullRequestStatusUseCase<'a> {
    #[tracing::instrument(
        skip_all,
        fields(
            repo_owner = %self.repo_owner,
            repo_name = %self.repo_name,
            pr_number = self.pr_number,
            head_sha = %self.upstream_pr.head.sha
        )
    )]
    pub async fn run(&mut self) -> Result<()> {
        let commit_sha = &self.upstream_pr.head.sha;
        let pr_status = BuildPullRequestStatusUseCase {
            api_service: self.api_service,
            db_service: self.db_service,
            pr_number: self.pr_number,
            repo_name: self.repo_name,
            repo_owner: self.repo_owner,
            upstream_pr: self.upstream_pr,
        }
        .run()
        .await?;

        // Update step label.
        let step_label = DetermineAutomaticStepUseCase {
            pr_status: &pr_status,
        }
        .run();

        self.apply_pull_request_step(Some(step_label)).await?;

        // Post status.
        PostSummaryCommentUseCase {
            api_service: self.api_service,
            db_service: self.db_service,
            lock_service: self.lock_service,
            repo_name: self.repo_name,
            repo_owner: self.repo_owner,
            pr_number: self.pr_number,
            pr_status: &pr_status,
        }
        .run()
        .await?;

        // Create or update status.
        let status_message = GenerateStatusMessageUseCase {
            pr_status: &pr_status,
        }
        .run()?;

        self.api_service
            .commit_statuses_update(
                self.repo_owner,
                self.repo_name,
                commit_sha,
                status_message.state,
                status_message.title,
                &status_message.message,
            )
            .await?;

        let pr_model = self
            .db_service
            .pull_requests_get(self.repo_owner, self.repo_name, self.pr_number)
            .await?
            .unwrap();

        // Merge if auto-merge is enabled
        if matches!(step_label, StepLabel::AwaitingMerge)
            && self.upstream_pr.merged != Some(true)
            && pr_model.automerge
        {
            // Use lock
            let key = format!(
                "pr-merge_{}-{}_{}",
                self.repo_owner, self.repo_name, self.pr_number
            );
            if let LockStatus::SuccessfullyLocked(l) =
                self.lock_service.try_lock_resource(&key).await?
            {
                if !self.try_automerge_pull_request().await? {
                    self.db_service
                        .pull_requests_set_automerge(
                            self.repo_owner,
                            self.repo_name,
                            self.pr_number,
                            false,
                        )
                        .await?;

                    // Update status
                    PostSummaryCommentUseCase {
                        api_service: self.api_service,
                        db_service: self.db_service,
                        lock_service: self.lock_service,
                        repo_name: self.repo_name,
                        repo_owner: self.repo_owner,
                        pr_number: self.pr_number,
                        pr_status: &pr_status,
                    }
                    .run()
                    .await?;
                }

                l.release().await?;
            }
        }

        Ok(())
    }

    /// Apply pull request step.
    #[tracing::instrument(skip(self))]
    async fn apply_pull_request_step(&mut self, step: Option<StepLabel>) -> Result<()> {
        SetStepLabelUseCase {
            api_service: self.api_service,
            repo_owner: self.repo_owner,
            repo_name: self.repo_name,
            pr_number: self.pr_number,
            label: step,
        }
        .run()
        .await
        .map_err(Into::into)
    }

    #[tracing::instrument(
        skip_all,
        fields(
            repo_owner = %self.repo_owner,
            repo_name = %self.repo_name,
            pr_number = self.pr_number
        ),
        ret
    )]
    async fn try_automerge_pull_request(&mut self) -> Result<bool> {
        let repository = self
            .db_service
            .repositories_get(self.repo_owner, self.repo_name)
            .await?
            .unwrap();
        let pull_request = self
            .db_service
            .pull_requests_get(self.repo_owner, self.repo_name, self.pr_number)
            .await?
            .unwrap();

        let strategy = if let Some(s) = pull_request.strategy_override {
            s
        } else {
            DeterminePullRequestMergeStrategyUseCase {
                db_service: self.db_service,
                repo_owner: self.repo_owner,
                repo_name: self.repo_name,
                head_branch: &self.upstream_pr.base.reference,
                base_branch: &self.upstream_pr.head.reference,
                default_strategy: repository.default_strategy,
            }
            .run()
            .await?
        };

        let merge_result = MergePullRequestUseCase {
            api_service: self.api_service,
            repo_name: self.repo_name,
            repo_owner: self.repo_owner,
            pr_number: self.pr_number,
            merge_strategy: strategy,
            upstream_pr: self.upstream_pr,
        }
        .run()
        .await;

        match merge_result {
            Ok(()) => {
                CommentApi::post_comment(
                    self.api_service,
                    self.repo_owner,
                    self.repo_name,
                    self.pr_number,
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
                    self.repo_owner,
                    self.repo_name,
                    self.pr_number,
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
            pr_number: 1,
            repo_owner: "me",
            repo_name: "test",
            upstream_pr: &upstream_pr,
        }
        .run()
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
            pr_number: 1,
            repo_owner: "me",
            repo_name: "test",
            upstream_pr: &upstream_pr,
        }
        .run()
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
            pr_number: 1,
            repo_owner: "me",
            repo_name: "test",
            upstream_pr: &upstream_pr,
        }
        .run()
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
