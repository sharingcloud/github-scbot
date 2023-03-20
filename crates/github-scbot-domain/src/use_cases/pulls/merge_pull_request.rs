use github_scbot_domain_models::{MergeStrategy, PullRequestHandle};
use github_scbot_ghapi_interface::{
    types::{GhMergeStrategy, GhPullRequest},
    ApiError, ApiService,
};

pub struct MergePullRequestUseCase<'a> {
    pub api_service: &'a dyn ApiService,
}

impl<'a> MergePullRequestUseCase<'a> {
    fn convert_strategy_for_github(strategy: MergeStrategy) -> GhMergeStrategy {
        match strategy {
            MergeStrategy::Merge => GhMergeStrategy::Merge,
            MergeStrategy::Rebase => GhMergeStrategy::Rebase,
            MergeStrategy::Squash => GhMergeStrategy::Squash,
        }
    }

    #[tracing::instrument(skip(self), fields(pr_handle, merge_strategy))]
    pub async fn run(
        &self,
        pr_handle: &PullRequestHandle,
        merge_strategy: MergeStrategy,
        upstream_pr: &GhPullRequest,
    ) -> Result<(), ApiError> {
        let commit_title = format!("{} (#{})", upstream_pr.title, upstream_pr.number);

        self.api_service
            .pulls_merge(
                pr_handle.repository().owner(),
                pr_handle.repository().name(),
                pr_handle.number(),
                &commit_title,
                "",
                Self::convert_strategy_for_github(merge_strategy),
            )
            .await
    }
}
