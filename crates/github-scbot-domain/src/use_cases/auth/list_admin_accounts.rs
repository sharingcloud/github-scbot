use github_scbot_database_interface::DbService;
use github_scbot_domain_models::Account;

use crate::Result;

pub struct ListAdminAccountsUseCase<'a> {
    pub db_service: &'a mut dyn DbService,
}

impl<'a> ListAdminAccountsUseCase<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn run(&mut self) -> Result<Vec<Account>> {
        self.db_service
            .accounts_list_admins()
            .await
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use github_scbot_database_interface::DbService;
    use github_scbot_database_memory::MemoryDb;
    use github_scbot_domain_models::Account;

    use super::ListAdminAccountsUseCase;

    #[tokio::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let mut db = MemoryDb::new();

        let one = db
            .accounts_create(Account {
                username: "one".into(),
                is_admin: true,
            })
            .await?;

        let two = db
            .accounts_create(Account {
                username: "two".into(),
                is_admin: true,
            })
            .await?;

        assert_eq!(
            ListAdminAccountsUseCase {
                db_service: &mut db,
            }
            .run()
            .await?,
            vec![one, two]
        );

        Ok(())
    }
}
