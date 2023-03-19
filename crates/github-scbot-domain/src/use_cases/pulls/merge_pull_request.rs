use github_scbot_domain_models::MergeStrategy;
use github_scbot_ghapi_interface::{
    types::{GhMergeStrategy, GhPullRequest},
    ApiError, ApiService,
};

pub struct MergePullRequestUseCase<'a> {
    pub api_service: &'a dyn ApiService,
    pub repo_name: &'a str,
    pub repo_owner: &'a str,
    pub pr_number: u64,
    pub merge_strategy: MergeStrategy,
    pub upstream_pr: &'a GhPullRequest,
}

impl<'a> MergePullRequestUseCase<'a> {
    fn convert_strategy_for_github(strategy: MergeStrategy) -> GhMergeStrategy {
        match strategy {
            MergeStrategy::Merge => GhMergeStrategy::Merge,
            MergeStrategy::Rebase => GhMergeStrategy::Rebase,
            MergeStrategy::Squash => GhMergeStrategy::Squash,
        }
    }

    #[tracing::instrument(skip(self), fields(self.repo_owner, self.repo_name, self.pr_number, self.merge_strategy))]
    pub async fn run(&mut self) -> Result<(), ApiError> {
        let commit_title = format!("{} (#{})", self.upstream_pr.title, self.upstream_pr.number);

        self.api_service
            .pulls_merge(
                self.repo_owner,
                self.repo_name,
                self.pr_number,
                &commit_title,
                "",
                Self::convert_strategy_for_github(self.merge_strategy),
            )
            .await
    }
}
