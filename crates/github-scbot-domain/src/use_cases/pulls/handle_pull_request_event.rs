use github_scbot_database_interface::DbService;
use github_scbot_ghapi_interface::{
    types::{GhPullRequestAction, GhPullRequestEvent},
    ApiService,
};

use crate::{use_cases::status::UpdatePullRequestStatusUseCaseInterface, Result};

pub struct HandlePullRequestEventUseCase<'a> {
    pub api_service: &'a dyn ApiService,
    pub db_service: &'a dyn DbService,
    pub update_pull_request_status: &'a dyn UpdatePullRequestStatusUseCaseInterface,
}

impl<'a> HandlePullRequestEventUseCase<'a> {
    #[tracing::instrument(
        skip_all,
        fields(
            action = ?event.action,
            pr_number = event.number,
            repository_path = %event.repository.full_name,
            username = %event.pull_request.user.login
        )
    )]
    pub async fn run(&self, event: GhPullRequestEvent) -> Result<()> {
        let repo_owner = &event.repository.owner.login;
        let repo_name = &event.repository.name;

        let pr_model = match self
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
            let upstream_pr = self
                .api_service
                .pulls_get(repo_owner, repo_name, pr_number)
                .await?;

            self.update_pull_request_status
                .run(
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
    use github_scbot_database_memory::MemoryDb;
    use github_scbot_domain_models::{PullRequest, Repository};
    use github_scbot_ghapi_interface::{
        types::{GhPullRequest, GhRepository, GhUser},
        MockApiService,
    };

    use super::*;
    use crate::use_cases::status::MockUpdatePullRequestStatusUseCaseInterface;

    #[tokio::test]
    async fn sync_event_on_unknown_pull_request_should_not_update_status() {
        let api_service = MockApiService::new();
        let db_service = MemoryDb::new();
        let update_pull_request_status = MockUpdatePullRequestStatusUseCaseInterface::new();

        HandlePullRequestEventUseCase {
            api_service: &api_service,
            db_service: &db_service,
            update_pull_request_status: &update_pull_request_status,
        }
        .run(GhPullRequestEvent {
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
        })
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn assigned_event_on_known_pull_request_should_not_update_status() {
        let api_service = MockApiService::new();
        let db_service = MemoryDb::new();
        let update_pull_request_status = MockUpdatePullRequestStatusUseCaseInterface::new();

        let repo = db_service
            .repositories_create(Repository {
                owner: "me".into(),
                name: "test".into(),
                ..Default::default()
            })
            .await
            .unwrap();

        db_service
            .pull_requests_create(
                PullRequest {
                    number: 1,
                    ..Default::default()
                }
                .with_repository(&repo),
            )
            .await
            .unwrap();

        HandlePullRequestEventUseCase {
            api_service: &api_service,
            db_service: &db_service,
            update_pull_request_status: &update_pull_request_status,
        }
        .run(GhPullRequestEvent {
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
        })
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn edited_event_on_known_pull_request_should_update_status() {
        let mut api_service = MockApiService::new();
        let db_service = MemoryDb::new();
        let mut update_pull_request_status = MockUpdatePullRequestStatusUseCaseInterface::new();

        api_service
            .expect_pulls_get()
            .once()
            .withf(|owner, name, number| owner == "me" && name == "test" && number == &1)
            .return_once(|_, _, _| {
                Ok(GhPullRequest {
                    number: 1,
                    ..Default::default()
                })
            });

        update_pull_request_status
            .expect_run()
            .once()
            .withf(|pr_handle, upstream_pr| {
                pr_handle == &("me", "test", 1).into() && upstream_pr.number == 1
            })
            .return_once(|_, _| Ok(()));

        let repo = db_service
            .repositories_create(Repository {
                owner: "me".into(),
                name: "test".into(),
                ..Default::default()
            })
            .await
            .unwrap();

        db_service
            .pull_requests_create(
                PullRequest {
                    number: 1,
                    ..Default::default()
                }
                .with_repository(&repo),
            )
            .await
            .unwrap();

        HandlePullRequestEventUseCase {
            api_service: &api_service,
            db_service: &db_service,
            update_pull_request_status: &update_pull_request_status,
        }
        .run(GhPullRequestEvent {
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
        })
        .await
        .unwrap()
    }
}
