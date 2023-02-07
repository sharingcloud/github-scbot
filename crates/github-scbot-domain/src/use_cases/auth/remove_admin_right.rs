use github_scbot_database::{Account, DbServiceAll};

use crate::Result;

pub struct RemoveAdminRightUseCase<'a> {
    pub username: String,
    pub db_service: &'a mut dyn DbServiceAll,
}

impl<'a> RemoveAdminRightUseCase<'a> {
    pub async fn run(&mut self) -> Result<()> {
        match self.db_service.accounts_get(&self.username).await? {
            Some(_) => {
                self.db_service
                    .accounts_set_is_admin(&self.username, false)
                    .await?
            }
            None => {
                self.db_service
                    .accounts_create(Account {
                        username: self.username.clone(),
                        is_admin: false,
                    })
                    .await?
            }
        };

        Ok(())
    }
}
