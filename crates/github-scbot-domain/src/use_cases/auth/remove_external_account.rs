use github_scbot_database_interface::DbService;

use crate::Result;

pub struct RemoveExternalAccountUseCase<'a> {
    pub username: String,
    pub db_service: &'a mut dyn DbService,
}

impl<'a> RemoveExternalAccountUseCase<'a> {
    pub async fn run(&mut self) -> Result<()> {
        self.db_service
            .external_accounts_delete(&self.username)
            .await?;

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
        let mut db = MemoryDb::new();

        db.external_accounts_create(ExternalAccount {
            username: "acc".into(),
            ..Default::default()
        })
        .await?;

        RemoveExternalAccountUseCase {
            username: "acc".into(),
            db_service: &mut db,
        }
        .run()
        .await?;

        assert_eq!(db.external_accounts_get("acc").await?, None);

        Ok(())
    }
}
