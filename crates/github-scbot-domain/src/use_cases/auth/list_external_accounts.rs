use github_scbot_database::{DbServiceAll, ExternalAccount};

use crate::Result;

pub struct ListExternalAccountsUseCase<'a> {
    pub db_service: &'a mut dyn DbServiceAll,
}

impl<'a> ListExternalAccountsUseCase<'a> {
    pub async fn run(&mut self) -> Result<Vec<ExternalAccount>> {
        self.db_service
            .external_accounts_all()
            .await
            .map_err(Into::into)
    }
}
