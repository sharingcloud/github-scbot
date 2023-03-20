use github_scbot_database_interface::DbService;
use github_scbot_domain_models::ExternalAccount;

use crate::Result;

pub struct ListExternalAccountsUseCase<'a> {
    pub db_service: &'a dyn DbService,
}

impl<'a> ListExternalAccountsUseCase<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn run(&mut self) -> Result<Vec<ExternalAccount>> {
        self.db_service
            .external_accounts_all()
            .await
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use github_scbot_database_interface::DbService;
    use github_scbot_database_memory::MemoryDb;
    use github_scbot_domain_models::ExternalAccount;

    use super::ListExternalAccountsUseCase;

    #[tokio::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let db = MemoryDb::new();

        let acc = db
            .external_accounts_create(ExternalAccount {
                username: "acc".into(),
                ..Default::default()
            })
            .await?;

        assert_eq!(
            ListExternalAccountsUseCase { db_service: &db }
                .run()
                .await?,
            vec![acc]
        );

        Ok(())
    }
}
