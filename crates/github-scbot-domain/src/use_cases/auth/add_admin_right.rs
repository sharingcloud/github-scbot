use github_scbot_database_interface::DbService;
use github_scbot_domain_models::Account;

use crate::Result;

pub struct AddAdminRightUseCase<'a> {
    pub username: &'a str,
    pub db_service: &'a dyn DbService,
}

impl<'a> AddAdminRightUseCase<'a> {
    #[tracing::instrument(skip(self), fields(self.username))]
    pub async fn run(&mut self) -> Result<()> {
        match self.db_service.accounts_get(self.username).await? {
            Some(_) => {
                self.db_service
                    .accounts_set_is_admin(self.username, true)
                    .await?
            }
            None => {
                self.db_service
                    .accounts_create(Account {
                        username: self.username.into(),
                        is_admin: true,
                    })
                    .await?
            }
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use github_scbot_database_interface::DbService;
    use github_scbot_database_memory::MemoryDb;

    use super::AddAdminRightUseCase;

    #[tokio::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let db = MemoryDb::new();

        AddAdminRightUseCase {
            username: "me",
            db_service: &db,
        }
        .run()
        .await?;

        assert!(db.accounts_get_expect("me").await?.is_admin);

        Ok(())
    }
}
