use async_trait::async_trait;
use github_scbot_domain_models::{MergeStrategy, PullRequestHandle};
use github_scbot_ghapi_interface::{
    types::{GhMergeStrategy, GhPullRequest},
    ApiError, ApiService,
};

#[mockall::automock]
#[async_trait(?Send)]
pub trait MergePullRequestUseCaseInterface {
    async fn run(
        &self,
        pr_handle: &PullRequestHandle,
        merge_strategy: MergeStrategy,
        upstream_pr: &GhPullRequest,
    ) -> Result<(), ApiError>;
}

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
}

#[async_trait(?Send)]
impl<'a> MergePullRequestUseCaseInterface for MergePullRequestUseCase<'a> {
    #[tracing::instrument(skip(self), fields(pr_handle, merge_strategy))]
    async fn run(
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

#[cfg(test)]
mod tests {
    use github_scbot_ghapi_interface::MockApiService;

    use super::*;

    #[tokio::test]
    async fn merge_success() {
        let mut api_service = MockApiService::new();
        api_service
            .expect_pulls_merge()
            .once()
            .withf(|owner, name, number, title, body, strategy| {
                owner == "me"
                    && name == "test"
                    && number == &1
                    && title == "Test (#1)"
                    && body.is_empty()
                    && *strategy == GhMergeStrategy::Merge
            })
            .return_once(|_, _, _, _, _, _| Ok(()));

        MergePullRequestUseCase {
            api_service: &api_service,
        }
        .run(
            &("me", "test", 1).into(),
            MergeStrategy::Merge,
            &GhPullRequest {
                number: 1,
                title: "Test".into(),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn merge_failure() {
        let mut api_service = MockApiService::new();
        api_service
            .expect_pulls_merge()
            .once()
            .withf(|owner, name, number, title, body, strategy| {
                owner == "me"
                    && name == "test"
                    && number == &1
                    && title == "Test (#1)"
                    && body.is_empty()
                    && *strategy == GhMergeStrategy::Merge
            })
            .return_once(|_, _, _, _, _, _| {
                Err(ApiError::MergeError {
                    pr_number: 1,
                    repository_path: "me/test".into(),
                })
            });

        let result = MergePullRequestUseCase {
            api_service: &api_service,
        }
        .run(
            &("me", "test", 1).into(),
            MergeStrategy::Merge,
            &GhPullRequest {
                number: 1,
                title: "Test".into(),
                ..Default::default()
            },
        )
        .await;

        match result {
            Err(ApiError::MergeError {
                pr_number,
                repository_path,
            }) => {
                assert_eq!(pr_number, 1);
                assert_eq!(repository_path, "me/test");
            }
            _ => panic!("Should error"),
        }
    }
}
