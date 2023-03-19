use github_scbot_database_interface::DbService;
use github_scbot_domain_models::Account;

use crate::Result;

pub struct RemoveAdminRightUseCase<'a> {
    pub username: String,
    pub db_service: &'a mut dyn DbService,
}

impl<'a> RemoveAdminRightUseCase<'a> {
    #[tracing::instrument(skip(self), fields(self.username))]
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

#[cfg(test)]
mod tests {
    use std::error::Error;

    use github_scbot_database_interface::DbService;
    use github_scbot_database_memory::MemoryDb;
    use github_scbot_domain_models::Account;

    use super::RemoveAdminRightUseCase;

    #[tokio::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let mut db = MemoryDb::new();

        db.accounts_create(Account {
            username: "acc".into(),
            is_admin: true,
        })
        .await?;

        RemoveAdminRightUseCase {
            username: "acc".into(),
            db_service: &mut db,
        }
        .run()
        .await?;

        assert!(!db.accounts_get_expect("acc").await?.is_admin);

        Ok(())
    }
}
