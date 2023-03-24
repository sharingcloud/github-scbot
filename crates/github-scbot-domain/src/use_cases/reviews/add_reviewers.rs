use async_trait::async_trait;
use github_scbot_database_interface::DbService;
use github_scbot_domain_models::{PullRequestHandle, RequiredReviewer};
use github_scbot_ghapi_interface::ApiService;

use super::filter_reviewers::{FilterReviewersUseCaseInterface, FilteredReviewers};
use crate::Result;

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait(?Send)]
pub trait AddReviewersUseCaseInterface {
    async fn run<'a>(
        &self,
        pr_handle: &PullRequestHandle,
        reviewers: &[String],
        required: bool,
    ) -> Result<FilteredReviewers>;
}

pub struct AddReviewersUseCase<'a> {
    pub api_service: &'a dyn ApiService,
    pub db_service: &'a dyn DbService,
    pub filter_reviewers: &'a dyn FilterReviewersUseCaseInterface,
}

#[async_trait(?Send)]
impl<'a> AddReviewersUseCaseInterface for AddReviewersUseCase<'a> {
    #[tracing::instrument(skip(self), ret)]
    async fn run<'b>(
        &self,
        pr_handle: &PullRequestHandle,
        reviewers: &[String],
        required: bool,
    ) -> Result<FilteredReviewers> {
        let filtered = self
            .filter_reviewers
            .run(pr_handle.repository(), reviewers)
            .await?;

        // The pull request should be already available in database
        let pr_model = self
            .db_service
            .pull_requests_get(
                pr_handle.repository().owner(),
                pr_handle.repository().name(),
                pr_handle.number(),
            )
            .await?
            .unwrap();

        if required {
            for reviewer in &filtered.allowed {
                self.create_required_reviewer(pr_handle, pr_model.id, reviewer)
                    .await?;
            }
        }

        self.add_reviewers_on_pull_request(pr_handle, &filtered.allowed)
            .await?;

        Ok(filtered)
    }
}

impl<'a> AddReviewersUseCase<'a> {
    async fn create_required_reviewer(
        &self,
        pr_handle: &PullRequestHandle,
        pr_model_id: u64,
        reviewer: &str,
    ) -> Result<()> {
        if self
            .db_service
            .required_reviewers_get(
                pr_handle.repository().owner(),
                pr_handle.repository().name(),
                pr_handle.number(),
                reviewer,
            )
            .await?
            .is_none()
        {
            self.db_service
                .required_reviewers_create(RequiredReviewer {
                    pull_request_id: pr_model_id,
                    username: reviewer.into(),
                })
                .await?;
        }

        Ok(())
    }

    async fn add_reviewers_on_pull_request(
        &self,
        pr_handle: &PullRequestHandle,
        reviewers: &[String],
    ) -> Result<()> {
        self.api_service
            .pull_reviewer_requests_add(
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
    use github_scbot_domain_models::{PullRequest, Repository};
    use github_scbot_ghapi_interface::MockApiService;

    use super::*;
    use crate::use_cases::reviews::MockFilterReviewersUseCaseInterface;

    #[tokio::test]
    async fn not_required() {
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

            svc.expect_pull_reviewer_requests_add()
                .once()
                .withf(|owner, name, number, reviewers| {
                    owner == "me"
                        && name == "test"
                        && number == &1
                        && reviewers == ["reviewer_with_rights".to_string()]
                })
                .return_once(|_, _, _, _| Ok(()));

            svc
        };

        let filtered_reviewers = FilteredReviewers {
            allowed: vec!["reviewer_with_rights".into()],
            rejected: vec!["reviewer_without_rights".into()],
        };

        let filter_reviewers = {
            let mut mock = MockFilterReviewersUseCaseInterface::new();
            let filtered_reviewers = filtered_reviewers.clone();

            mock.expect_run()
                .once()
                .withf(|repository_path, reviewers| {
                    repository_path == &("me", "test").into() && reviewers.len() == 2
                })
                .return_once(move |_, _| Ok(filtered_reviewers));

            mock
        };

        let result = AddReviewersUseCase {
            api_service: &api_service,
            db_service: &db_service,
            filter_reviewers: &filter_reviewers,
        }
        .run(
            &("me", "test", 1).into(),
            &[
                "reviewer_with_rights".into(),
                "reviewer_without_rights".into(),
            ],
            false,
        )
        .await
        .unwrap();

        assert_eq!(result, filtered_reviewers);

        // Not marked as required
        assert_eq!(
            db_service
                .required_reviewers_list("me", "test", 1)
                .await
                .unwrap(),
            vec![]
        );
    }

    #[tokio::test]
    async fn required() {
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

            svc.expect_pull_reviewer_requests_add()
                .once()
                .withf(|owner, name, number, reviewers| {
                    owner == "me"
                        && name == "test"
                        && number == &1
                        && reviewers == ["reviewer_with_rights".to_string()]
                })
                .return_once(|_, _, _, _| Ok(()));

            svc
        };

        let filtered_reviewers = FilteredReviewers {
            allowed: vec!["reviewer_with_rights".into()],
            rejected: vec!["reviewer_without_rights".into()],
        };

        let filter_reviewers = {
            let mut mock = MockFilterReviewersUseCaseInterface::new();
            let filtered_reviewers = filtered_reviewers.clone();

            mock.expect_run()
                .once()
                .withf(|repository_path, reviewers| {
                    repository_path == &("me", "test").into() && reviewers.len() == 2
                })
                .return_once(move |_, _| Ok(filtered_reviewers));

            mock
        };

        let result = AddReviewersUseCase {
            api_service: &api_service,
            db_service: &db_service,
            filter_reviewers: &filter_reviewers,
        }
        .run(
            &("me", "test", 1).into(),
            &[
                "reviewer_with_rights".into(),
                "reviewer_without_rights".into(),
            ],
            true,
        )
        .await
        .unwrap();

        assert_eq!(result, filtered_reviewers);

        // Marked as required
        assert_eq!(
            db_service
                .required_reviewers_list("me", "test", 1)
                .await
                .unwrap(),
            vec![RequiredReviewer {
                pull_request_id: 1,
                username: "reviewer_with_rights".into()
            }]
        );
    }
}
