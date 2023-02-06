use github_scbot_database::DbServiceAll;

use crate::Result;

pub struct RemoveExternalAccountUseCase<'a> {
    pub username: String,
    pub db_service: &'a mut dyn DbServiceAll,
}

impl<'a> RemoveExternalAccountUseCase<'a> {
    pub async fn run(&mut self) -> Result<()> {
        self.db_service
            .external_accounts_delete(&self.username)
            .await?;

        Ok(())
    }
}
