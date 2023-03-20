use github_scbot_database_interface::DbService;

use crate::Result;

pub struct GenerateExternalAccountTokenUseCase<'a> {
    pub db_service: &'a dyn DbService,
}

impl<'a> GenerateExternalAccountTokenUseCase<'a> {
    #[tracing::instrument(skip(self), fields(self.username))]
    pub async fn run(&self, username: &str) -> Result<String> {
        let exa = self
            .db_service
            .external_accounts_get(username)
            .await?
            .unwrap();
        exa.generate_access_token().map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use github_scbot_database_interface::DbService;
    use github_scbot_database_memory::MemoryDb;
    use github_scbot_domain_models::ExternalAccount;

    use super::GenerateExternalAccountTokenUseCase;

    #[tokio::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let db = MemoryDb::new();

        db.external_accounts_create(
            ExternalAccount {
                username: "me".into(),
                ..Default::default()
            }
            .with_generated_keys(),
        )
        .await?;

        assert!(GenerateExternalAccountTokenUseCase { db_service: &db }
            .run("me")
            .await?
            .starts_with("ey"));

        Ok(())
    }
}
