use github_scbot_core::types::repository::RepositoryPath;
use github_scbot_database::DbServiceAll;

use crate::Result;

pub struct RemoveExternalAccountRightUseCase<'a> {
    pub username: String,
    pub repository_path: RepositoryPath,
    pub db_service: &'a mut dyn DbServiceAll,
}

impl<'a> RemoveExternalAccountRightUseCase<'a> {
    pub async fn run(&mut self) -> Result<()> {
        let (owner, name) = self.repository_path.components();

        self.db_service
            .external_account_rights_delete(owner, name, &self.username)
            .await?;

        Ok(())
    }
}
