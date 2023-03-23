use async_trait::async_trait;
use github_scbot_database_interface::DbService;
use github_scbot_domain_models::PullRequestHandle;
use github_scbot_ghapi_interface::{comments::CommentApi, types::GhPullRequest, ApiService};

use super::{DeterminePullRequestMergeStrategyUseCase, MergePullRequestUseCaseInterface};
use crate::Result;

#[mockall::automock]
#[async_trait(?Send)]
pub trait AutomergePullRequestUseCaseInterface {
    async fn run(&self, pr_handle: &PullRequestHandle, upstream_pr: &GhPullRequest)
        -> Result<bool>;
}

pub struct AutomergePullRequestUseCase<'a> {
    pub db_service: &'a dyn DbService,
    pub api_service: &'a dyn ApiService,
    pub merge_pull_request: &'a dyn MergePullRequestUseCaseInterface,
}

#[async_trait(?Send)]
impl<'a> AutomergePullRequestUseCaseInterface for AutomergePullRequestUseCase<'a> {
    async fn run(
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

        let merge_result = self
            .merge_pull_request
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
    use github_scbot_domain_models::{MergeStrategy, PullRequest, Repository};
    use github_scbot_ghapi_interface::{ApiError, MockApiService};

    use super::*;
    use crate::use_cases::pulls::MockMergePullRequestUseCaseInterface;

    #[tokio::test]
    async fn run_success() {
        let db_service = {
            let svc = MemoryDb::new();
            let repo = svc
                .repositories_create(Repository {
                    owner: "me".into(),
                    name: "test".into(),
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

        let api_service = {
            let mut svc = MockApiService::new();
            svc.expect_comments_post()
                .once()
                .withf(|owner, name, number, body| {
                    owner == "me" && name == "test" && number == &1 && !body.is_empty()
                })
                .return_once(|_, _, _, _| Ok(1));

            svc
        };

        let merge_pull_request = {
            let mut mock = MockMergePullRequestUseCaseInterface::new();
            mock.expect_run()
                .once()
                .withf(|handle, strategy, upstream_pr| {
                    handle == &("me", "test", 1).into()
                        && *strategy == MergeStrategy::Merge
                        && upstream_pr.number == 1
                })
                .return_once(|_, _, _| Ok(()));

            mock
        };

        assert!(AutomergePullRequestUseCase {
            api_service: &api_service,
            db_service: &db_service,
            merge_pull_request: &merge_pull_request
        }
        .run(
            &("me", "test", 1).into(),
            &GhPullRequest {
                number: 1,
                ..Default::default()
            }
        )
        .await
        .unwrap());
    }

    #[tokio::test]
    async fn run_failure() {
        let db_service = {
            let svc = MemoryDb::new();
            let repo = svc
                .repositories_create(Repository {
                    owner: "me".into(),
                    name: "test".into(),
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

        let api_service = {
            let mut svc = MockApiService::new();
            svc.expect_comments_post()
                .once()
                .withf(|owner, name, number, body| {
                    owner == "me" && name == "test" && number == &1 && !body.is_empty()
                })
                .return_once(|_, _, _, _| Ok(1));

            svc
        };

        let merge_pull_request = {
            let mut mock = MockMergePullRequestUseCaseInterface::new();
            mock.expect_run()
                .once()
                .withf(|handle, strategy, upstream_pr| {
                    handle == &("me", "test", 1).into()
                        && *strategy == MergeStrategy::Merge
                        && upstream_pr.number == 1
                })
                .return_once(|_, _, _| {
                    Err(ApiError::MergeError {
                        pr_number: 1,
                        repository_path: "me/test".into(),
                    })
                });

            mock
        };

        assert!(!AutomergePullRequestUseCase {
            api_service: &api_service,
            db_service: &db_service,
            merge_pull_request: &merge_pull_request
        }
        .run(
            &("me", "test", 1).into(),
            &GhPullRequest {
                number: 1,
                ..Default::default()
            }
        )
        .await
        .unwrap());
    }
}
