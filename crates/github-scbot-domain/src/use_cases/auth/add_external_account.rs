use github_scbot_database::{DbServiceAll, ExternalAccount};

use crate::Result;

pub struct AddExternalAccountUseCase<'a> {
    pub username: String,
    pub db_service: &'a mut dyn DbServiceAll,
}

impl<'a> AddExternalAccountUseCase<'a> {
    pub async fn run(&mut self) -> Result<()> {
        self.db_service
            .external_accounts_create(
                ExternalAccount {
                    username: self.username.clone(),
                    ..Default::default()
                }
                .with_generated_keys(),
            )
            .await?;

        Ok(())
    }
}
