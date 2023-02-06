use github_scbot_core::types::repository::RepositoryPath;
use github_scbot_database::{DbServiceAll, ExternalAccountRight};

use crate::Result;

pub struct AddExternalAccountRightUseCase<'a> {
    pub repository_path: RepositoryPath,
    pub username: &'a str,
    pub db_service: &'a mut dyn DbServiceAll,
}

impl<'a> AddExternalAccountRightUseCase<'a> {
    pub async fn run(&mut self) -> Result<()> {
        let (owner, name) = self.repository_path.components();
        let repository = self
            .db_service
            .repositories_get(owner, name)
            .await?
            .expect("unknown repository");

        self.db_service
            .external_account_rights_delete(owner, name, self.username)
            .await?;
        self.db_service
            .external_account_rights_create(
                ExternalAccountRight::builder()
                    .repository_id(repository.id())
                    .username(self.username)
                    .build()
                    .unwrap(),
            )
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::AddExternalAccountRightUseCase;
    use github_scbot_database::{DbServiceAll, ExternalAccount, MemoryDb, Repository};

    #[actix_rt::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let mut db_service = MemoryDb::new();
        let repository = db_service
            .repositories_create(Repository::builder().owner("owner").name("name").build()?)
            .await?;
        db_service
            .external_accounts_create(ExternalAccount::builder().username("me").build()?)
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