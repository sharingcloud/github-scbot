use async_trait::async_trait;
use prbot_models::{PullRequestHandle, RequiredReviewer};
use shaku::{Component, HasComponent, Interface};

use super::filter_reviewers::{FilterReviewersInterface, FilteredReviewers};
use crate::{CoreContext, Result};

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait]
pub trait AddReviewersInterface: Interface {
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        pr_handle: &PullRequestHandle,
        reviewers: &[String],
        required: bool,
    ) -> Result<FilteredReviewers>;
}

#[derive(Component)]
#[shaku(interface = AddReviewersInterface)]
pub(crate) struct AddReviewers;

#[async_trait]
impl AddReviewersInterface for AddReviewers {
    #[tracing::instrument(skip(self, ctx), ret)]
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        pr_handle: &PullRequestHandle,
        reviewers: &[String],
        required: bool,
    ) -> Result<FilteredReviewers> {
        let filter_reviewers: &dyn FilterReviewersInterface = ctx.core_module.resolve_ref();
        let filtered = filter_reviewers
            .run(ctx, pr_handle.repository_path(), reviewers)
            .await?;

        // The pull request should be already available in database
        let pr_model = ctx
            .db_service
            .pull_requests_get(
                pr_handle.repository_path().owner(),
                pr_handle.repository_path().name(),
                pr_handle.number(),
            )
            .await?
            .unwrap();

        if required {
            for reviewer in &filtered.allowed {
                self.create_required_reviewer(ctx, pr_handle, pr_model.id, reviewer)
                    .await?;
            }
        }

        self.add_reviewers_on_pull_request(ctx, pr_handle, &filtered.allowed)
            .await?;

        Ok(filtered)
    }
}

impl AddReviewers {
    async fn create_required_reviewer(
        &self,
        ctx: &CoreContext<'_>,
        pr_handle: &PullRequestHandle,
        pr_model_id: u64,
        reviewer: &str,
    ) -> Result<()> {
        if ctx
            .db_service
            .required_reviewers_get(
                pr_handle.repository_path().owner(),
                pr_handle.repository_path().name(),
                pr_handle.number(),
                reviewer,
            )
            .await?
            .is_none()
        {
            ctx.db_service
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
        ctx: &CoreContext<'_>,
        pr_handle: &PullRequestHandle,
        reviewers: &[String],
    ) -> Result<()> {
        ctx.api_service
            .pull_reviewer_requests_add(
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
    use prbot_models::{PullRequest, Repository};

    use super::*;
    use crate::{
        context::tests::CoreContextTest, use_cases::reviews::MockFilterReviewersInterface,
        CoreModule,
    };

    #[tokio::test]
    async fn not_required() {
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

            svc
        };

        ctx.api_service = {
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
            let mut mock = MockFilterReviewersInterface::new();
            let filtered_reviewers = filtered_reviewers.clone();

            mock.expect_run()
                .once()
                .withf(|_, repository_path, reviewers| {
                    repository_path == &("me", "test").into() && reviewers.len() == 2
                })
                .return_once(move |_, _, _| Ok(filtered_reviewers));

            mock
        };

        ctx.core_module = CoreModule::builder()
            .with_component_override::<dyn FilterReviewersInterface>(Box::new(filter_reviewers))
            .build();

        let result = AddReviewers
            .run(
                &ctx.as_context(),
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
            ctx.db_service
                .required_reviewers_list("me", "test", 1)
                .await
                .unwrap(),
            vec![]
        );
    }

    #[tokio::test]
    async fn required() {
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

            svc
        };

        ctx.api_service = {
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
            let mut mock = MockFilterReviewersInterface::new();
            let filtered_reviewers = filtered_reviewers.clone();

            mock.expect_run()
                .once()
                .withf(|_, repository_path, reviewers| {
                    repository_path == &("me", "test").into() && reviewers.len() == 2
                })
                .return_once(move |_, _, _| Ok(filtered_reviewers));

            mock
        };

        ctx.core_module = CoreModule::builder()
            .with_component_override::<dyn FilterReviewersInterface>(Box::new(filter_reviewers))
            .build();

        let result = AddReviewers
            .run(
                &ctx.as_context(),
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
            ctx.db_service
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
