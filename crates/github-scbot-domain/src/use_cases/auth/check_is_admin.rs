use github_scbot_database::DbServiceAll;

use crate::Result;

pub struct CheckIsAdminUseCase<'a> {
    pub username: &'a str,
    pub db_service: &'a mut dyn DbServiceAll,
}

impl<'a> CheckIsAdminUseCase<'a> {
    pub async fn run(&mut self) -> Result<bool> {
        let known_admins: Vec<_> = self.db_service.accounts_list_admins().await?;

        Ok(known_admins.iter().any(|acc| acc.username == self.username))
    }
}
