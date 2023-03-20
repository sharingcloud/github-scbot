use github_scbot_database_interface::DbService;

use crate::Result;

pub struct RemoveExternalAccountUseCase<'a> {
    pub db_service: &'a dyn DbService,
}

impl<'a> RemoveExternalAccountUseCase<'a> {
    #[tracing::instrument(skip(self), fields(username))]
    pub async fn run(&self, username: &str) -> Result<()> {
        self.db_service.external_accounts_delete(username).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use github_scbot_database_interface::DbService;
    use github_scbot_database_memory::MemoryDb;
    use github_scbot_domain_models::ExternalAccount;

    use super::RemoveExternalAccountUseCase;

    #[tokio::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let db = MemoryDb::new();

        db.external_accounts_create(ExternalAccount {
            username: "acc".into(),
            ..Default::default()
        })
        .await?;

        RemoveExternalAccountUseCase { db_service: &db }
            .run("acc")
            .await?;

        assert_eq!(db.external_accounts_get("acc").await?, None);

        Ok(())
    }
}
