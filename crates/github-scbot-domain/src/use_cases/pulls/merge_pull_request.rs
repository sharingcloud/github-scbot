use github_scbot_core::types::pulls::{GhMergeStrategy, GhPullRequest};
use github_scbot_ghapi_interface::{ApiError, ApiService};

pub struct MergePullRequestUseCase<'a> {
    pub api_service: &'a dyn ApiService,
    pub repo_name: &'a str,
    pub repo_owner: &'a str,
    pub pr_number: u64,
    pub merge_strategy: GhMergeStrategy,
    pub upstream_pr: &'a GhPullRequest,
}

impl<'a> MergePullRequestUseCase<'a> {
    pub async fn run(&mut self) -> Result<(), ApiError> {
        let commit_title = format!("{} (#{})", self.upstream_pr.title, self.upstream_pr.number);

        self.api_service
            .pulls_merge(
                self.repo_owner,
                self.repo_name,
                self.pr_number,
                &commit_title,
                "",
                self.merge_strategy,
            )
            .await
    }
}
