use github_scbot_config::Config;
use github_scbot_database_interface::DbService;
use github_scbot_domain_models::Repository;

use crate::Result;

pub struct GetOrCreateRepositoryUseCase<'a> {
    pub db_service: &'a mut dyn DbService,
    pub repo_name: &'a str,
    pub repo_owner: &'a str,
    pub config: &'a Config,
}

impl<'a> GetOrCreateRepositoryUseCase<'a> {
    pub async fn run(&mut self) -> Result<Repository> {
        match self
            .db_service
            .repositories_get(self.repo_owner, self.repo_name)
            .await?
        {
            Some(r) => Ok(r),
            None => Ok(self
                .db_service
                .repositories_create(
                    Repository {
                        owner: self.repo_owner.into(),
                        name: self.repo_name.into(),
                        ..Default::default()
                    }
                    .with_config(self.config),
                )
                .await?),
        }
    }
}
