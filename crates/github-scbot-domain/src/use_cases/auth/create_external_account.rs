use github_scbot_database::{DbServiceAll, ExternalAccount};

use crate::Result;

pub struct CreateExternalAccountUseCase<'a> {
    pub username: String,
    pub db_service: &'a mut dyn DbServiceAll,
}

impl<'a> CreateExternalAccountUseCase<'a> {
    pub async fn run(&mut self) -> Result<()> {
        self.db_service
            .external_accounts_create(
                ExternalAccount::builder()
                    .username(self.username.clone())
                    .generate_keys()
                    .build()
                    .unwrap(),
            )
            .await?;

        Ok(())
    }
}
