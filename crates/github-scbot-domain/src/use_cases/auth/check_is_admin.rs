use github_scbot_database_interface::DbService;

use crate::Result;

pub struct CheckIsAdminUseCase<'a> {
    pub db_service: &'a dyn DbService,
}

impl<'a> CheckIsAdminUseCase<'a> {
    #[tracing::instrument(skip(self), fields(self.username), ret)]
    pub async fn run(&self, username: &str) -> Result<bool> {
        let known_admins: Vec<_> = self.db_service.accounts_list_admins().await?;

        Ok(known_admins.iter().any(|acc| acc.username == username))
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use github_scbot_database_interface::DbService;
    use github_scbot_database_memory::MemoryDb;
    use github_scbot_domain_models::Account;

    use super::CheckIsAdminUseCase;

    #[tokio::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let db = MemoryDb::new();

        db.accounts_create(Account {
            username: "me".into(),
            is_admin: true,
        })
        .await?;

        assert!(CheckIsAdminUseCase { db_service: &db }.run("me").await?);

        Ok(())
    }

    #[tokio::test]
    async fn run_not_admin() -> Result<(), Box<dyn Error>> {
        let db = MemoryDb::new();

        db.accounts_create(Account {
            username: "me".into(),
            is_admin: false,
        })
        .await?;

        assert!(!CheckIsAdminUseCase { db_service: &db }.run("me").await?);

        Ok(())
    }
}
