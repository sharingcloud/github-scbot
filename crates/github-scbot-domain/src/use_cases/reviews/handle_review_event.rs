use github_scbot_database_interface::DbService;
use github_scbot_ghapi_interface::{types::GhReviewEvent, ApiService};

use crate::{use_cases::status::UpdatePullRequestStatusUseCaseInterface, Result};

pub struct HandleReviewEventUseCase<'a> {
    pub api_service: &'a dyn ApiService,
    pub db_service: &'a dyn DbService,
    pub update_pull_request_status: &'a dyn UpdatePullRequestStatusUseCaseInterface,
}

impl<'a> HandleReviewEventUseCase<'a> {
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
    pub async fn run(&self, event: GhReviewEvent) -> Result<()> {
        let repo_owner = &event.repository.owner.login;
        let repo_name = &event.repository.name;
        let pr_number = event.pull_request.number;

        // Detect required reviews
        if self
            .db_service
            .pull_requests_get(repo_owner, repo_name, pr_number)
            .await?
            .is_some()
        {
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
    async fn run_unknown_pull_request() {
        let api_service = MockApiService::new();
        let db_service = MemoryDb::new();
        let update_pull_request_status = MockUpdatePullRequestStatusUseCaseInterface::new();

        HandleReviewEventUseCase {
            api_service: &api_service,
            db_service: &db_service,
            update_pull_request_status: &update_pull_request_status,
        }
        .run(GhReviewEvent {
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
    async fn run_known_pull_request() {
        let mut api_service = MockApiService::new();
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

        let db_service = MemoryDb::new();
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

        let mut update_pull_request_status = MockUpdatePullRequestStatusUseCaseInterface::new();
        update_pull_request_status
            .expect_run()
            .once()
            .withf(|pr_handle, upstream_pr| {
                pr_handle == &("me", "test", 1).into() && upstream_pr.number == 1
            })
            .return_once(|_, _| Ok(()));

        HandleReviewEventUseCase {
            api_service: &api_service,
            db_service: &db_service,
            update_pull_request_status: &update_pull_request_status,
        }
        .run(GhReviewEvent {
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
