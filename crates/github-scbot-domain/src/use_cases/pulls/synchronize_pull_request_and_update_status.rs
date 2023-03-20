use github_scbot_config::Config;
use github_scbot_database_interface::DbService;
use github_scbot_domain_models::PullRequestHandle;
use github_scbot_ghapi_interface::ApiService;
use github_scbot_lock_interface::LockService;

use super::SynchronizePullRequestUseCase;
use crate::{use_cases::status::UpdatePullRequestStatusUseCase, Result};

pub struct SynchronizePullRequestAndUpdateStatusUseCase<'a> {
    pub config: &'a Config,
    pub db_service: &'a dyn DbService,
    pub api_service: &'a dyn ApiService,
    pub lock_service: &'a dyn LockService,
}

impl<'a> SynchronizePullRequestAndUpdateStatusUseCase<'a> {
    #[tracing::instrument(skip(self), fields(pr_handle))]
    pub async fn run(&self, pr_handle: &PullRequestHandle) -> Result<()> {
        SynchronizePullRequestUseCase {
            config: self.config,
            db_service: self.db_service,
        }
        .run(pr_handle)
        .await?;

        let upstream_pr = self
            .api_service
            .pulls_get(
                pr_handle.repository().owner(),
                pr_handle.repository().name(),
                pr_handle.number(),
            )
            .await?;

        UpdatePullRequestStatusUseCase {
            api_service: self.api_service,
            db_service: self.db_service,
            lock_service: self.lock_service,
        }
        .run(pr_handle, &upstream_pr)
        .await?;

        Ok(())
    }
}
