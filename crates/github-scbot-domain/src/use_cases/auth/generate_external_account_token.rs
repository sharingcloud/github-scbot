use github_scbot_database_interface::DbService;

use crate::Result;

pub struct GenerateExternalAccountTokenUseCase<'a> {
    pub username: &'a str,
    pub db_service: &'a mut dyn DbService,
}

impl<'a> GenerateExternalAccountTokenUseCase<'a> {
    pub async fn run(&mut self) -> Result<String> {
        let exa = self
            .db_service
            .external_accounts_get(self.username)
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
        let mut db = MemoryDb::new();

        db.external_accounts_create(
            ExternalAccount {
                username: "me".into(),
                ..Default::default()
            }
            .with_generated_keys(),
        )
        .await?;

        assert!(GenerateExternalAccountTokenUseCase {
            username: "me",
            db_service: &mut db,
        }
        .run()
        .await?
        .starts_with("ey"));

        Ok(())
    }
}
