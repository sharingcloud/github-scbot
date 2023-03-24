use async_trait::async_trait;
use github_scbot_domain_models::RepositoryPath;
use github_scbot_ghapi_interface::ApiService;

use crate::Result;

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait(?Send)]
pub trait FilterReviewersUseCaseInterface {
    async fn run(
        &self,
        repository_path: &RepositoryPath,
        reviewers: &[String],
    ) -> Result<FilteredReviewers>;
}

pub struct FilterReviewersUseCase<'a> {
    pub api_service: &'a dyn ApiService,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FilteredReviewers {
    pub allowed: Vec<String>,
    pub rejected: Vec<String>,
}

#[async_trait(?Send)]
impl<'a> FilterReviewersUseCaseInterface for FilterReviewersUseCase<'a> {
    #[tracing::instrument(skip(self), ret)]
    async fn run(
        &self,
        repository_path: &RepositoryPath,
        reviewers: &[String],
    ) -> Result<FilteredReviewers> {
        let (mut allowed, mut rejected) = (vec![], vec![]);

        for reviewer in reviewers {
            let permission = self
                .api_service
                .user_permissions_get(repository_path.owner(), repository_path.name(), reviewer)
                .await?
                .can_write();

            if permission {
                allowed.push(reviewer.clone());
            } else {
                rejected.push(reviewer.clone());
            }
        }

        Ok(FilteredReviewers { allowed, rejected })
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_ghapi_interface::{types::GhUserPermission, MockApiService};

    use super::*;

    #[tokio::test]
    async fn right_and_no_right() {
        let api_service = {
            let mut svc = MockApiService::new();

            svc.expect_user_permissions_get()
                .once()
                .withf(|owner, name, user| {
                    owner == "me" && name == "test" && user == "reviewer_with_rights"
                })
                .return_once(|_, _, _| Ok(GhUserPermission::Write));

            svc.expect_user_permissions_get()
                .once()
                .withf(|owner, name, user| {
                    owner == "me" && name == "test" && user == "reviewer_without_rights"
                })
                .return_once(|_, _, _| Ok(GhUserPermission::None));

            svc
        };

        let result = FilterReviewersUseCase {
            api_service: &api_service,
        }
        .run(
            &("me", "test").into(),
            &[
                "reviewer_with_rights".into(),
                "reviewer_without_rights".into(),
            ],
        )
        .await
        .unwrap();

        assert_eq!(
            result,
            FilteredReviewers {
                allowed: vec!["reviewer_with_rights".into()],
                rejected: vec!["reviewer_without_rights".into()],
            }
        )
    }
}
