use github_scbot_database::{DbService, ExternalAccount};

use crate::Result;

pub struct AddExternalAccountUseCase<'a> {
    pub username: &'a str,
    pub db_service: &'a mut dyn DbService,
}

impl<'a> AddExternalAccountUseCase<'a> {
    pub async fn run(&mut self) -> Result<()> {
        self.db_service
            .external_accounts_create(
                ExternalAccount {
                    username: self.username.into(),
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

    use github_scbot_database::{DbService, MemoryDb};

    use super::AddExternalAccountUseCase;

    #[actix_rt::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let mut db = MemoryDb::new();

        AddExternalAccountUseCase {
            username: "me",
            db_service: &mut db,
        }
        .run()
        .await?;

        assert!(db.external_accounts_get("me").await?.is_some());

        Ok(())
    }
}
