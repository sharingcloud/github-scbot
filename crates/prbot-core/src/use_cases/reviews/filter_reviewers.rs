use async_trait::async_trait;
use prbot_models::RepositoryPath;
use shaku::{Component, Interface};

use crate::{CoreContext, Result};

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait]
pub trait FilterReviewersInterface: Interface {
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        repository_path: &RepositoryPath,
        reviewers: &[String],
    ) -> Result<FilteredReviewers>;
}

#[derive(Component)]
#[shaku(interface = FilterReviewersInterface)]
pub(crate) struct FilterReviewers;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FilteredReviewers {
    pub allowed: Vec<String>,
    pub rejected: Vec<String>,
}

#[async_trait]
impl FilterReviewersInterface for FilterReviewers {
    #[tracing::instrument(skip(self, ctx), ret)]
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        repository_path: &RepositoryPath,
        reviewers: &[String],
    ) -> Result<FilteredReviewers> {
        let (mut allowed, mut rejected) = (vec![], vec![]);

        for reviewer in reviewers {
            let permission = ctx
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
    use prbot_ghapi_interface::{types::GhUserPermission, MockApiService};

    use super::*;
    use crate::context::tests::CoreContextTest;

    #[tokio::test]
    async fn right_and_no_right() {
        let mut ctx = CoreContextTest::new();

        ctx.api_service = {
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

        let result = FilterReviewers
            .run(
                &ctx.as_context(),
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
