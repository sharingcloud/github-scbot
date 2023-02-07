use github_scbot_database::{DbServiceAll, Repository};

use crate::Result;

pub struct ListExternalAccountRightsUseCase<'a> {
    pub username: String,
    pub db_service: &'a mut dyn DbServiceAll,
}

impl<'a> ListExternalAccountRightsUseCase<'a> {
    pub async fn run(&mut self) -> Result<Vec<Repository>> {
        let rights = self
            .db_service
            .external_account_rights_list(&self.username)
            .await?;

        let mut repositories = Vec::new();
        if !rights.is_empty() {
            for right in rights {
                repositories.push(
                    self.db_service
                        .repositories_get_from_id(right.repository_id)
                        .await?
                        .unwrap(),
                );
            }
        }

        Ok(repositories)
    }
}
