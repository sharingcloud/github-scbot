use github_scbot_database_interface::DbService;
use github_scbot_domain_models::{ExternalAccountRight, RepositoryPath};

use crate::Result;

pub struct AddExternalAccountRightUseCase<'a> {
    pub db_service: &'a dyn DbService,
}

impl<'a> AddExternalAccountRightUseCase<'a> {
    #[tracing::instrument(skip(self), fields(repository_path, username))]
    pub async fn run(&self, repository_path: &RepositoryPath, username: &str) -> Result<()> {
        let (owner, name) = repository_path.components();
        let repository = self.db_service.repositories_get_expect(owner, name).await?;

        self.db_service
            .external_account_rights_delete(owner, name, username)
            .await?;
        self.db_service
            .external_account_rights_create(ExternalAccountRight {
                repository_id: repository.id,
                username: username.into(),
            })
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use github_scbot_database_interface::DbService;
    use github_scbot_database_memory::MemoryDb;
    use github_scbot_domain_models::{ExternalAccount, Repository};

    use super::AddExternalAccountRightUseCase;

    #[tokio::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let db_service = MemoryDb::new();
        let repository = db_service
            .repositories_create(Repository {
                owner: "owner".into(),
                name: "name".into(),
                ..Default::default()
            })
            .await?;
        db_service
            .external_accounts_create(ExternalAccount {
                username: "me".into(),
                ..Default::default()
            })
            .await?;

        AddExternalAccountRightUseCase {
            db_service: &db_service,
        }
        .run(&repository.path(), "me")
        .await?;

        assert!(db_service
            .external_account_rights_get("owner", "name", "me")
            .await?
            .is_some());

        Ok(())
    }
}
