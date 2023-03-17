use github_scbot_core::types::{labels::StepLabel, pulls::GhPullRequest};
use github_scbot_database_interface::DbService;
use github_scbot_ghapi_interface::{comments::CommentApi, labels::LabelApi, ApiService};
use github_scbot_lock_interface::{LockService, LockStatus};

use crate::{
    use_cases::{
        pulls::{DeterminePullRequestMergeStrategyUseCase, MergePullRequestUseCase},
        summary::PostSummaryCommentUseCase,
    },
    Result,
};

use super::{
    build_pull_request_status::BuildPullRequestStatusUseCase,
    determine_automatic_step::DetermineAutomaticStepUseCase,
    generate_status_message::GenerateStatusMessageUseCase,
};

pub struct UpdatePullRequestStatusUseCase<'a> {
    pub api_service: &'a dyn ApiService,
    pub db_service: &'a mut dyn DbService,
    pub redis_service: &'a dyn LockService,
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
            redis_service: self.redis_service,
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
                self.redis_service.try_lock_resource(&key).await?
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
                        redis_service: self.redis_service,
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
        LabelApi::set_step_label(
            self.api_service,
            self.repo_owner,
            self.repo_name,
            self.pr_number,
            step,
        )
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
