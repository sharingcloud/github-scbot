use github_scbot_database::DbServiceAll;

use crate::Result;

pub struct CreateExternalTokenUseCase<'a> {
    pub username: String,
    pub db_service: &'a mut dyn DbServiceAll,
}

impl<'a> CreateExternalTokenUseCase<'a> {
    pub async fn run(&mut self) -> Result<String> {
        let exa = self
            .db_service
            .external_accounts_get(&self.username)
            .await?
            .unwrap();
        exa.generate_access_token().map_err(Into::into)
    }
}
