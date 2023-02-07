use github_scbot_database::DbServiceAll;

use crate::Result;

pub struct GenerateExternalTokenUseCase<'a> {
    pub username: &'a str,
    pub db_service: &'a mut dyn DbServiceAll,
}

impl<'a> GenerateExternalTokenUseCase<'a> {
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

    use github_scbot_database::{DbServiceAll, ExternalAccount, MemoryDb};

    use super::GenerateExternalTokenUseCase;

    #[actix_rt::test]
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

        assert!(GenerateExternalTokenUseCase {
            username: "me",
            db_service: &mut db,
        }
        .run()
        .await?
        .starts_with("ey"));

        Ok(())
    }
}
