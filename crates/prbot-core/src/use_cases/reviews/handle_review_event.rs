use async_trait::async_trait;
use prbot_ghapi_interface::types::GhReviewEvent;
use shaku::{Component, HasComponent, Interface};

use crate::{use_cases::status::UpdatePullRequestStatusInterface, CoreContext, Result};

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait]
pub trait HandleReviewEventInterface: Interface {
    async fn run<'a>(&self, ctx: &CoreContext<'a>, event: GhReviewEvent) -> Result<()>;
}

#[derive(Component)]
#[shaku(interface = HandleReviewEventInterface)]
pub(crate) struct HandleReviewEvent;

#[async_trait]
impl HandleReviewEventInterface for HandleReviewEvent {
    #[tracing::instrument(
        skip_all,
        fields(
            repo_owner = event.repository.owner.login,
            repo_name = event.repository.name,
            pr_number = event.pull_request.number,
            reviewer = event.review.user.login,
            state = ?event.review.state
        )
    )]
    async fn run<'a>(&self, ctx: &CoreContext<'a>, event: GhReviewEvent) -> Result<()> {
        let repo_owner = &event.repository.owner.login;
        let repo_name = &event.repository.name;
        let pr_number = event.pull_request.number;

        // Detect required reviews
        if ctx
            .db_service
            .pull_requests_get(repo_owner, repo_name, pr_number)
            .await?
            .is_some()
        {
            let upstream_pr = ctx
                .api_service
                .pulls_get(repo_owner, repo_name, pr_number)
                .await?;

            let update_pull_request_status: &dyn UpdatePullRequestStatusInterface =
                ctx.core_module.resolve_ref();
            update_pull_request_status
                .run(
                    ctx,
                    &(repo_owner.as_str(), repo_name.as_str(), pr_number).into(),
                    &upstream_pr,
                )
                .await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use prbot_database_interface::DbService;
    use prbot_ghapi_interface::types::{GhPullRequest, GhRepository, GhReviewEvent, GhUser};
    use prbot_models::{PullRequest, Repository};

    use super::HandleReviewEvent;
    use crate::{
        context::tests::CoreContextTest,
        use_cases::{
            reviews::handle_review_event::HandleReviewEventInterface,
            status::{MockUpdatePullRequestStatusInterface, UpdatePullRequestStatusInterface},
        },
        CoreModule,
    };

    #[tokio::test]
    async fn run_unknown_pull_request() {
        let ctx = CoreContextTest::new();

        HandleReviewEvent
            .run(
                &ctx.as_context(),
                GhReviewEvent {
                    pull_request: GhPullRequest {
                        number: 1,
                        ..Default::default()
                    },
                    repository: GhRepository {
                        owner: GhUser { login: "me".into() },
                        name: "test".into(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn run_known_pull_request() {
        let mut ctx = CoreContextTest::new();
        ctx.api_service
            .expect_pulls_get()
            .once()
            .withf(|owner, name, number| owner == "me" && name == "test" && number == &1)
            .return_once(|_, _, _| {
                Ok(GhPullRequest {
                    number: 1,
                    ..Default::default()
                })
            });

        let repo = ctx
            .db_service
            .repositories_create(Repository {
                owner: "me".into(),
                name: "test".into(),
                ..Default::default()
            })
            .await
            .unwrap();
        ctx.db_service
            .pull_requests_create(
                PullRequest {
                    number: 1,
                    ..Default::default()
                }
                .with_repository(&repo),
            )
            .await
            .unwrap();

        let mut update_pull_request_status = MockUpdatePullRequestStatusInterface::new();
        update_pull_request_status
            .expect_run()
            .once()
            .withf(|_, pr_handle, upstream_pr| {
                pr_handle == &("me", "test", 1).into() && upstream_pr.number == 1
            })
            .return_once(|_, _, _| Ok(()));

        ctx.core_module = CoreModule::builder()
            .with_component_override::<dyn UpdatePullRequestStatusInterface>(Box::new(
                update_pull_request_status,
            ))
            .build();

        HandleReviewEvent
            .run(
                &ctx.as_context(),
                GhReviewEvent {
                    pull_request: GhPullRequest {
                        number: 1,
                        ..Default::default()
                    },
                    repository: GhRepository {
                        owner: GhUser { login: "me".into() },
                        name: "test".into(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .await
            .unwrap()
    }
}
