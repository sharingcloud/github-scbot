use async_trait::async_trait;
use prbot_models::PullRequestHandle;
use shaku::{Component, Interface};

use crate::{CoreContext, Result};

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait]
pub trait RemoveReviewersInterface: Interface {
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        pr_handle: &PullRequestHandle,
        reviewers: &[String],
    ) -> Result<()>;
}

#[derive(Component)]
#[shaku(interface = RemoveReviewersInterface)]
pub struct RemoveReviewers;

#[async_trait]
impl RemoveReviewersInterface for RemoveReviewers {
    #[tracing::instrument(skip(self, ctx), ret)]
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        pr_handle: &PullRequestHandle,
        reviewers: &[String],
    ) -> Result<()> {
        for reviewer in reviewers {
            // Just in case, cleanup required reviewers
            self.remove_required_reviewer(ctx, pr_handle, reviewer)
                .await?;
        }

        self.remove_reviewers_on_pull_request(ctx, pr_handle, reviewers)
            .await?;

        Ok(())
    }
}

impl RemoveReviewers {
    async fn remove_required_reviewer(
        &self,
        ctx: &CoreContext<'_>,
        pr_handle: &PullRequestHandle,
        reviewer: &str,
    ) -> Result<()> {
        ctx.db_service
            .required_reviewers_delete(
                pr_handle.repository_path().owner(),
                pr_handle.repository_path().name(),
                pr_handle.number(),
                reviewer,
            )
            .await?;

        Ok(())
    }

    async fn remove_reviewers_on_pull_request(
        &self,
        ctx: &CoreContext<'_>,
        pr_handle: &PullRequestHandle,
        reviewers: &[String],
    ) -> Result<()> {
        ctx.api_service
            .pull_reviewer_requests_remove(
                pr_handle.repository_path().owner(),
                pr_handle.repository_path().name(),
                pr_handle.number(),
                reviewers,
            )
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use prbot_database_interface::DbService;
    use prbot_database_memory::MemoryDb;
    use prbot_ghapi_interface::MockApiService;
    use prbot_models::{PullRequest, Repository, RequiredReviewer};

    use super::*;
    use crate::context::tests::CoreContextTest;

    #[tokio::test]
    async fn run() {
        let mut ctx = CoreContextTest::new();
        ctx.db_service = {
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

        ctx.api_service = {
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

        RemoveReviewers
            .run(
                &ctx.as_context(),
                &("me", "test", 1).into(),
                &[
                    "reviewer_with_rights".into(),
                    "reviewer_without_rights".into(),
                ],
            )
            .await
            .unwrap();

        assert_eq!(
            ctx.db_service
                .required_reviewers_list("me", "test", 1)
                .await
                .unwrap(),
            vec![]
        );
    }
}
