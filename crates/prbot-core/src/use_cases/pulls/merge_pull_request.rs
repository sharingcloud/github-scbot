use async_trait::async_trait;
use prbot_ghapi_interface::{
    types::{GhMergeStrategy, GhPullRequest},
    ApiError,
};
use prbot_models::{MergeStrategy, PullRequestHandle};
use shaku::{Component, Interface};

use crate::CoreContext;

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait]
pub trait MergePullRequestInterface: Interface {
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        pr_handle: &PullRequestHandle,
        merge_strategy: MergeStrategy,
        upstream_pr: &GhPullRequest,
    ) -> Result<(), ApiError>;
}

#[derive(Component)]
#[shaku(interface = MergePullRequestInterface)]
pub(crate) struct MergePullRequest;

impl MergePullRequest {
    fn convert_strategy_for_github(strategy: MergeStrategy) -> GhMergeStrategy {
        match strategy {
            MergeStrategy::Merge => GhMergeStrategy::Merge,
            MergeStrategy::Rebase => GhMergeStrategy::Rebase,
            MergeStrategy::Squash => GhMergeStrategy::Squash,
        }
    }
}

#[async_trait]
impl MergePullRequestInterface for MergePullRequest {
    #[tracing::instrument(skip(self, ctx), fields(pr_handle, merge_strategy))]
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        pr_handle: &PullRequestHandle,
        merge_strategy: MergeStrategy,
        upstream_pr: &GhPullRequest,
    ) -> Result<(), ApiError> {
        let commit_title = format!("{} (#{})", upstream_pr.title, upstream_pr.number);

        ctx.api_service
            .pulls_merge(
                pr_handle.repository_path().owner(),
                pr_handle.repository_path().name(),
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
    use super::*;
    use crate::context::tests::CoreContextTest;

    #[tokio::test]
    async fn merge_success() {
        let mut ctx = CoreContextTest::new();
        ctx.api_service
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

        MergePullRequest
            .run(
                &ctx.as_context(),
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
        let mut ctx = CoreContextTest::new();
        ctx.api_service
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

        let result = MergePullRequest
            .run(
                &ctx.as_context(),
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
