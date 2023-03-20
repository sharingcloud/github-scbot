use github_scbot_database_interface::DbService;
use github_scbot_domain_models::ExternalAccount;

use crate::Result;

pub struct AddExternalAccountUseCase<'a> {
    pub db_service: &'a dyn DbService,
}

impl<'a> AddExternalAccountUseCase<'a> {
    #[tracing::instrument(skip(self), fields(self.username))]
    pub async fn run(&self, username: &str) -> Result<()> {
        self.db_service
            .external_accounts_create(
                ExternalAccount {
                    username: username.into(),
                    ..Default::default()
                }
                .with_generated_keys(),
            )
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use github_scbot_database_interface::DbService;
    use github_scbot_database_memory::MemoryDb;

    use super::AddExternalAccountUseCase;

    #[tokio::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let db = MemoryDb::new();

        AddExternalAccountUseCase { db_service: &db }
            .run("me")
            .await?;

        assert!(db.external_accounts_get("me").await?.is_some());

        Ok(())
    }
}
