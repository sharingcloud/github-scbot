use github_scbot_core::types::repository::RepositoryPath;
use github_scbot_database_interface::DbService;
use github_scbot_domain_models::ExternalAccountRight;

use crate::Result;

pub struct AddExternalAccountRightUseCase<'a> {
    pub repository_path: RepositoryPath,
    pub username: &'a str,
    pub db_service: &'a mut dyn DbService,
}

impl<'a> AddExternalAccountRightUseCase<'a> {
    pub async fn run(&mut self) -> Result<()> {
        let (owner, name) = self.repository_path.components();
        let repository = self.db_service.repositories_get_expect(owner, name).await?;

        self.db_service
            .external_account_rights_delete(owner, name, self.username)
            .await?;
        self.db_service
            .external_account_rights_create(ExternalAccountRight {
                repository_id: repository.id,
                username: self.username.into(),
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

    #[actix_rt::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let mut db_service = MemoryDb::new();
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
            repository_path: repository.path(),
            username: "me",
            db_service: &mut db_service,
        }
        .run()
        .await?;

        assert!(db_service
            .external_account_rights_get("owner", "name", "me")
            .await?
            .is_some());

        Ok(())
    }
}
