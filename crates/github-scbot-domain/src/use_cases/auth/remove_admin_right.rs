use github_scbot_database_interface::DbService;
use github_scbot_domain_models::Account;

use crate::Result;

pub struct RemoveAdminRightUseCase<'a> {
    pub db_service: &'a dyn DbService,
}

impl<'a> RemoveAdminRightUseCase<'a> {
    #[tracing::instrument(skip(self), fields(username))]
    pub async fn run(&self, username: &str) -> Result<()> {
        match self.db_service.accounts_get(username).await? {
            Some(_) => {
                self.db_service
                    .accounts_set_is_admin(username, false)
                    .await?
            }
            None => {
                self.db_service
                    .accounts_create(Account {
                        username: username.to_string(),
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
        let db = MemoryDb::new();

        db.accounts_create(Account {
            username: "acc".into(),
            is_admin: true,
        })
        .await?;

        RemoveAdminRightUseCase { db_service: &db }
            .run("acc")
            .await?;

        assert!(!db.accounts_get_expect("acc").await?.is_admin);

        Ok(())
    }
}
