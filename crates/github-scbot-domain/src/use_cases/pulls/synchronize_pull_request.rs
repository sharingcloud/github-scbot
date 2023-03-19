use github_scbot_config::Config;
use github_scbot_database_interface::DbService;
use github_scbot_domain_models::PullRequest;

use super::GetOrCreateRepositoryUseCase;
use crate::Result;

pub struct SynchronizePullRequestUseCase<'a> {
    pub config: &'a Config,
    pub db_service: &'a mut dyn DbService,
    pub repo_owner: &'a str,
    pub repo_name: &'a str,
    pub pr_number: u64,
}

impl<'a> SynchronizePullRequestUseCase<'a> {
    #[tracing::instrument(skip(self), fields(self.repo_owner, self.repo_name, self.pr_number))]
    pub async fn run(&mut self) -> Result<()> {
        let repo = GetOrCreateRepositoryUseCase {
            db_service: self.db_service,
            config: self.config,
            repo_name: self.repo_name,
            repo_owner: self.repo_owner,
        }
        .run()
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

#[cfg(test)]
mod tests {
    use github_scbot_database_memory::MemoryDb;
    use github_scbot_domain_models::{QaStatus, Repository};

    use super::*;

    #[tokio::test]
    async fn synchronize() {
        let mut config = Config::from_env();
        config.default_needed_reviewers_count = 0;

        let mut db_service = MemoryDb::new();

        SynchronizePullRequestUseCase {
            db_service: &mut db_service,
            config: &config,
            repo_owner: "me",
            repo_name: "test",
            pr_number: 1,
        }
        .run()
        .await
        .unwrap();

        assert_eq!(
            db_service.repositories_all().await.unwrap(),
            vec![Repository {
                id: 1,
                owner: "me".into(),
                name: "test".into(),
                default_needed_reviewers_count: 0,
                default_enable_checks: true,
                default_enable_qa: false,
                ..Default::default()
            }]
        );

        assert_eq!(
            db_service.pull_requests_all().await.unwrap(),
            vec![PullRequest {
                id: 1,
                number: 1,
                repository_id: 1,
                needed_reviewers_count: 0,
                checks_enabled: true,
                qa_status: QaStatus::Skipped,
                ..Default::default()
            }]
        );
    }
}
