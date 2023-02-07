use github_scbot_database::DbServiceAll;

use crate::Result;

pub struct RemoveAllExternalAccountRightsUseCase<'a> {
    pub username: String,
    pub db_service: &'a mut dyn DbServiceAll,
}

impl<'a> RemoveAllExternalAccountRightsUseCase<'a> {
    pub async fn run(&mut self) -> Result<()> {
        self.db_service
            .external_account_rights_delete_all(&self.username)
            .await?;

        Ok(())
    }
}
