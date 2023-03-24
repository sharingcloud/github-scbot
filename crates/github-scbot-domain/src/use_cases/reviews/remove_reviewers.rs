use async_trait::async_trait;
use github_scbot_database_interface::DbService;
use github_scbot_domain_models::PullRequestHandle;
use github_scbot_ghapi_interface::ApiService;

use crate::Result;

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait(?Send)]
pub trait RemoveReviewersUseCaseInterface {
    async fn run<'a>(&self, pr_handle: &PullRequestHandle, reviewers: &[String]) -> Result<()>;
}

pub struct RemoveReviewersUseCase<'a> {
    pub api_service: &'a dyn ApiService,
    pub db_service: &'a dyn DbService,
}

#[async_trait(?Send)]
impl<'a> RemoveReviewersUseCaseInterface for RemoveReviewersUseCase<'a> {
    #[tracing::instrument(skip(self), ret)]
    async fn run<'b>(&self, pr_handle: &PullRequestHandle, reviewers: &[String]) -> Result<()> {
        for reviewer in reviewers {
            // Just in case, cleanup required reviewers
            self.remove_required_reviewer(pr_handle, reviewer).await?;
        }

        self.remove_reviewers_on_pull_request(pr_handle, reviewers)
            .await?;

        Ok(())
    }
}

impl<'a> RemoveReviewersUseCase<'a> {
    async fn remove_required_reviewer(
        &self,
        pr_handle: &PullRequestHandle,
        reviewer: &str,
    ) -> Result<()> {
        self.db_service
            .required_reviewers_delete(
                pr_handle.repository().owner(),
                pr_handle.repository().name(),
                pr_handle.number(),
                reviewer,
            )
            .await?;

        Ok(())
    }

    async fn remove_reviewers_on_pull_request(
        &self,
        pr_handle: &PullRequestHandle,
        reviewers: &[String],
    ) -> Result<()> {
        self.api_service
            .pull_reviewer_requests_remove(
                pr_handle.repository().owner(),
                pr_handle.repository().name(),
                pr_handle.number(),
                reviewers,
            )
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_database_memory::MemoryDb;
    use github_scbot_domain_models::{PullRequest, Repository, RequiredReviewer};
    use github_scbot_ghapi_interface::MockApiService;

    use super::*;

    #[tokio::test]
    async fn run() {
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

            svc.required_reviewers_create(RequiredReviewer {
                pull_request_id: 1,
                username: "reviewer_with_rights".into(),
            })
            .await
            .unwrap();

            svc
        };

        let api_service = {
            let mut svc = MockApiService::new();

            svc.expect_pull_reviewer_requests_remove()
                .once()
                .withf(|owner, name, number, reviewers| {
                    owner == "me"
                        && name == "test"
                        && number == &1
                        && reviewers
                            == [
                                "reviewer_with_rights".to_string(),
                                "reviewer_without_rights".to_string(),
                            ]
                })
                .return_once(|_, _, _, _| Ok(()));

            svc
        };

        RemoveReviewersUseCase {
            api_service: &api_service,
            db_service: &db_service,
        }
        .run(
            &("me", "test", 1).into(),
            &[
                "reviewer_with_rights".into(),
                "reviewer_without_rights".into(),
            ],
        )
        .await
        .unwrap();

        assert_eq!(
            db_service
                .required_reviewers_list("me", "test", 1)
                .await
                .unwrap(),
            vec![]
        );
    }
}
