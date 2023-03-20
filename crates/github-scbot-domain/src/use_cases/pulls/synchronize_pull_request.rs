use github_scbot_config::Config;
use github_scbot_database_interface::DbService;
use github_scbot_domain_models::{PullRequest, PullRequestHandle};

use super::GetOrCreateRepositoryUseCase;
use crate::Result;

pub struct SynchronizePullRequestUseCase<'a> {
    pub config: &'a Config,
    pub db_service: &'a dyn DbService,
}

impl<'a> SynchronizePullRequestUseCase<'a> {
    #[tracing::instrument(skip(self), fields(pr_handle))]
    pub async fn run(&self, pr_handle: &PullRequestHandle) -> Result<()> {
        let repo = GetOrCreateRepositoryUseCase {
            db_service: self.db_service,
            config: self.config,
        }
        .run(pr_handle.repository())
        .await?;

        if self
            .db_service
            .pull_requests_get(
                pr_handle.repository().owner(),
                pr_handle.repository().name(),
                pr_handle.number(),
            )
            .await?
            .is_none()
        {
            self.db_service
                .pull_requests_create(
                    PullRequest {
                        number: pr_handle.number(),
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

        let db_service = MemoryDb::new();

        SynchronizePullRequestUseCase {
            db_service: &db_service,
            config: &config,
        }
        .run(&("me", "test", 1).into())
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
