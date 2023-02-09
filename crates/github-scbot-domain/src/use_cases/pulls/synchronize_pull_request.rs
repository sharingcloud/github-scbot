use github_scbot_core::config::Config;
use github_scbot_database_interface::DbService;
use github_scbot_domain_models::PullRequest;

use crate::{pulls::PullRequestLogic, Result};

pub struct SynchronizePullRequestUseCase<'a> {
    pub config: &'a Config,
    pub db_service: &'a mut dyn DbService,
    pub repo_owner: &'a str,
    pub repo_name: &'a str,
    pub pr_number: u64,
}

impl<'a> SynchronizePullRequestUseCase<'a> {
    pub async fn run(&mut self) -> Result<()> {
        let repo = PullRequestLogic::get_or_create_repository(
            self.config,
            self.db_service,
            self.repo_owner,
            self.repo_name,
        )
        .await?;

        if self
            .db_service
            .pull_requests_get(self.repo_owner, self.repo_name, self.pr_number)
            .await?
            .is_none()
        {
            self.db_service
                .pull_requests_create(
                    PullRequest {
                        number: self.pr_number,
                        ..Default::default()
                    }
                    .with_repository(&repo),
                )
                .await?;
        }

        Ok(())
    }
}
