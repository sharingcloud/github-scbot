use github_scbot_domain_models::PullRequestHandle;
use github_scbot_ghapi_interface::ApiService;

use super::SynchronizePullRequestUseCaseInterface;
use crate::{use_cases::status::UpdatePullRequestStatusUseCaseInterface, Result};

pub struct SynchronizePullRequestAndUpdateStatusUseCase<'a> {
    pub api_service: &'a dyn ApiService,
    pub synchronize_pull_request: &'a dyn SynchronizePullRequestUseCaseInterface,
    pub update_pull_request_status: &'a dyn UpdatePullRequestStatusUseCaseInterface,
}

impl<'a> SynchronizePullRequestAndUpdateStatusUseCase<'a> {
    #[tracing::instrument(skip(self), fields(pr_handle))]
    pub async fn run(&self, pr_handle: &PullRequestHandle) -> Result<()> {
        self.synchronize_pull_request.run(pr_handle).await?;

        let upstream_pr = self
            .api_service
            .pulls_get(
                pr_handle.repository().owner(),
                pr_handle.repository().name(),
                pr_handle.number(),
            )
            .await?;

        self.update_pull_request_status
            .run(pr_handle, &upstream_pr)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_ghapi_interface::{types::GhPullRequest, MockApiService};

    use super::*;
    use crate::use_cases::{
        pulls::MockSynchronizePullRequestUseCaseInterface,
        status::MockUpdatePullRequestStatusUseCaseInterface,
    };

    #[tokio::test]
    async fn run() {
        let mut api_service = MockApiService::new();
        let mut synchronize_pull_request = MockSynchronizePullRequestUseCaseInterface::new();
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

        synchronize_pull_request
            .expect_run()
            .once()
            .withf(|pr_handle| pr_handle == &("me", "test", 1).into())
            .return_once(|_| Ok(()));

        update_pull_request_status
            .expect_run()
            .once()
            .withf(|pr_handle, upstream_pr| {
                pr_handle == &("me", "test", 1).into() && upstream_pr.number == 1
            })
            .return_once(|_, _| Ok(()));

        SynchronizePullRequestAndUpdateStatusUseCase {
            api_service: &api_service,
            synchronize_pull_request: &synchronize_pull_request,
            update_pull_request_status: &update_pull_request_status,
        }
        .run(&("me", "test", 1).into())
        .await
        .unwrap();
    }
}
