use github_scbot_config::Config;
use github_scbot_database_interface::DbService;
use github_scbot_domain_models::{Repository, RepositoryPath};

use crate::Result;

pub struct GetOrCreateRepositoryUseCase<'a> {
    pub db_service: &'a dyn DbService,
    pub config: &'a Config,
}

impl<'a> GetOrCreateRepositoryUseCase<'a> {
    #[tracing::instrument(skip(self), fields(repository_path))]
    pub async fn run(&self, repository_path: &RepositoryPath) -> Result<Repository> {
        match self
            .db_service
            .repositories_get(repository_path.owner(), repository_path.name())
            .await?
        {
            Some(r) => Ok(r),
            None => Ok(self
                .db_service
                .repositories_create(
                    Repository {
                        owner: repository_path.owner().into(),
                        name: repository_path.name().into(),
                        ..Default::default()
                    }
                    .with_config(self.config),
                )
                .await?),
        }
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_database_memory::MemoryDb;

    use super::*;

    #[tokio::test]
    async fn non_existing() {
        let config = Config::from_env();
        let db_service = MemoryDb::new();

        let repository = GetOrCreateRepositoryUseCase {
            db_service: &db_service,
            config: &config,
        }
        .run(&("me", "test").into())
        .await
        .unwrap();

        assert_eq!(repository.owner, "me");
        assert_eq!(repository.name, "test");
    }

    #[tokio::test]
    async fn already_existing() {
        let config = Config::from_env();
        let db_service = MemoryDb::new();
        let original_repo = db_service
            .repositories_create(Repository {
                owner: "me".into(),
                name: "test".into(),
                ..Default::default()
            })
            .await
            .unwrap();

        let repository = GetOrCreateRepositoryUseCase {
            db_service: &db_service,
            config: &config,
        }
        .run(&("me", "test").into())
        .await
        .unwrap();

        assert_eq!(original_repo, repository);
    }
}
