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
