use github_scbot_database::{Account, DbServiceAll};

use crate::Result;

pub struct ListAdminAccountsUseCase<'a> {
    pub db_service: &'a mut dyn DbServiceAll,
}

impl<'a> ListAdminAccountsUseCase<'a> {
    pub async fn run(&mut self) -> Result<Vec<Account>> {
        self.db_service
            .accounts_list_admins()
            .await
            .map_err(Into::into)
    }
}
