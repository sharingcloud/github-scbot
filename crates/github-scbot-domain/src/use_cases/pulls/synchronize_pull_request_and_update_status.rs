use github_scbot_config::Config;
use github_scbot_database_interface::DbService;
use github_scbot_ghapi_interface::ApiService;
use github_scbot_lock_interface::LockService;

use super::SynchronizePullRequestUseCase;
use crate::{use_cases::status::UpdatePullRequestStatusUseCase, Result};

pub struct SynchronizePullRequestAndUpdateStatusUseCase<'a> {
    pub config: &'a Config,
    pub db_service: &'a dyn DbService,
    pub api_service: &'a dyn ApiService,
    pub lock_service: &'a dyn LockService,
    pub repo_owner: &'a str,
    pub repo_name: &'a str,
    pub pr_number: u64,
}

impl<'a> SynchronizePullRequestAndUpdateStatusUseCase<'a> {
    #[tracing::instrument(skip(self), fields(self.repo_owner, self.repo_name, self.pr_number))]
    pub async fn run(&mut self) -> Result<()> {
        SynchronizePullRequestUseCase {
            config: self.config,
            db_service: self.db_service,
            repo_owner: self.repo_owner,
            repo_name: self.repo_name,
            pr_number: self.pr_number,
        }
        .run()
        .await?;

        let upstream_pr = self
            .api_service
            .pulls_get(self.repo_owner, self.repo_name, self.pr_number)
            .await?;

        UpdatePullRequestStatusUseCase {
            api_service: self.api_service,
            db_service: self.db_service,
            lock_service: self.lock_service,
            repo_owner: self.repo_owner,
            repo_name: self.repo_name,
            pr_number: self.pr_number,
            upstream_pr: &upstream_pr,
        }
        .run()
        .await?;

        Ok(())
    }
}
