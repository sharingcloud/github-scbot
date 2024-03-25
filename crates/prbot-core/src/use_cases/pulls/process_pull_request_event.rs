use async_trait::async_trait;
use prbot_ghapi_interface::types::{GhPullRequestAction, GhPullRequestEvent};
use shaku::{Component, HasComponent, Interface};

use super::ProcessPullRequestOpenedInterface;
use crate::{use_cases::status::UpdatePullRequestStatusInterface, CoreContext, Result};

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait]
pub trait ProcessPullRequestEventInterface: Interface {
    async fn run<'a>(&self, ctx: &CoreContext<'a>, gh_event: GhPullRequestEvent) -> Result<()>;
}

#[derive(Component)]
#[shaku(interface = ProcessPullRequestEventInterface)]
pub(crate) struct ProcessPullRequestEvent;

#[async_trait]
impl ProcessPullRequestEventInterface for ProcessPullRequestEvent {
    #[tracing::instrument(
        skip_all,
        fields(
            action = ?event.action,
            pr_number = event.number,
            repository_path = %event.repository.full_name,
            username = %event.pull_request.user.login
        )
    )]
    async fn run<'a>(&self, ctx: &CoreContext<'a>, event: GhPullRequestEvent) -> Result<()> {
        if event.action == GhPullRequestAction::Opened {
            let process_pull_request_opened: &dyn ProcessPullRequestOpenedInterface =
                ctx.core_module.resolve_ref();
            process_pull_request_opened.run(ctx, event).await?;

            return Ok(());
        }

        let repo_owner = &event.repository.owner.login;
        let repo_name = &event.repository.name;

        let pr_model = match ctx
            .db_service
            .pull_requests_get(repo_owner, repo_name, event.pull_request.number)
            .await?
        {
            Some(pr) => pr,
            None => return Ok(()),
        };

        let pr_number = pr_model.number;
        let mut status_changed = false;

        // Status update
        match event.action {
            GhPullRequestAction::Synchronize => {
                // Force status to waiting
                status_changed = true;
            }
            GhPullRequestAction::Reopened
            | GhPullRequestAction::ReadyForReview
            | GhPullRequestAction::ConvertedToDraft
            | GhPullRequestAction::Closed => {
                status_changed = true;
            }
            GhPullRequestAction::ReviewRequested => {
                status_changed = true;
            }
            GhPullRequestAction::ReviewRequestRemoved => {
                status_changed = true;
            }
            _ => (),
        }

        if let GhPullRequestAction::Edited = event.action {
            // Update PR title
            status_changed = true;
        }

        if status_changed {
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
    use prbot_ghapi_interface::types::{GhPullRequest, GhRepository, GhUser};
    use prbot_models::{PullRequest, Repository};

    use super::*;
    use crate::{
        context::tests::CoreContextTest, use_cases::status::MockUpdatePullRequestStatusInterface,
        CoreModule,
    };

    #[tokio::test]
    async fn sync_event_on_unknown_pull_request_should_not_update_status() {
        let ctx = CoreContextTest::new();

        ProcessPullRequestEvent
            .run(
                &ctx.as_context(),
                GhPullRequestEvent {
                    action: GhPullRequestAction::Synchronize,
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
    async fn assigned_event_on_known_pull_request_should_not_update_status() {
        let ctx = CoreContextTest::new();

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

        ProcessPullRequestEvent
            .run(
                &ctx.as_context(),
                GhPullRequestEvent {
                    action: GhPullRequestAction::Assigned,
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
    async fn edited_event_on_known_pull_request_should_update_status() {
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

        let mut update_pull_request_status = MockUpdatePullRequestStatusInterface::new();
        update_pull_request_status
            .expect_run()
            .once()
            .withf(|_, pr_handle, upstream_pr| {
                pr_handle == &("me", "test", 1).into() && upstream_pr.number == 1
            })
            .return_once(|_, _, _| Ok(()));

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

        ctx.core_module = CoreModule::builder()
            .with_component_override::<dyn UpdatePullRequestStatusInterface>(Box::new(
                update_pull_request_status,
            ))
            .build();

        ProcessPullRequestEvent
            .run(
                &ctx.as_context(),
                GhPullRequestEvent {
                    action: GhPullRequestAction::Edited,
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
