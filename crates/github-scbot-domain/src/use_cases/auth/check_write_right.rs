use github_scbot_database_interface::DbService;
use github_scbot_ghapi_interface::types::GhUserPermission;

use super::check_is_admin::CheckIsAdminUseCase;
use crate::Result;

pub struct CheckWriteRightUseCase<'a> {
    pub db_service: &'a dyn DbService,
}

impl<'a> CheckWriteRightUseCase<'a> {
    #[tracing::instrument(skip(self), fields(username, user_permission), ret)]
    pub async fn run(&self, username: &str, user_permission: GhUserPermission) -> Result<bool> {
        let is_admin = CheckIsAdminUseCase {
            db_service: self.db_service,
        }
        .run(username)
        .await?;

        Ok(is_admin || user_permission.can_write())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use github_scbot_database_interface::DbService;
    use github_scbot_database_memory::MemoryDb;
    use github_scbot_domain_models::Account;
    use github_scbot_ghapi_interface::types::GhUserPermission;

    use super::CheckWriteRightUseCase;

    #[tokio::test]
    async fn run_read_not_admin() -> Result<(), Box<dyn Error>> {
        let db = MemoryDb::new();

        db.accounts_create(Account {
            username: "me".into(),
            is_admin: false,
        })
        .await?;

        assert!(
            !CheckWriteRightUseCase { db_service: &db }
                .run("me", GhUserPermission::Read)
                .await?
        );

        Ok(())
    }

    #[tokio::test]
    async fn run_read_admin() -> Result<(), Box<dyn Error>> {
        let db = MemoryDb::new();

        db.accounts_create(Account {
            username: "me".into(),
            is_admin: true,
        })
        .await?;

        assert!(
            CheckWriteRightUseCase { db_service: &db }
                .run("me", GhUserPermission::Read)
                .await?
        );

        Ok(())
    }

    #[tokio::test]
    async fn run_write_admin() -> Result<(), Box<dyn Error>> {
        let db = MemoryDb::new();

        db.accounts_create(Account {
            username: "me".into(),
            is_admin: true,
        })
        .await?;

        assert!(
            CheckWriteRightUseCase { db_service: &db }
                .run("me", GhUserPermission::Write)
                .await?
        );

        Ok(())
    }

    #[tokio::test]
    async fn run_write_not_admin() -> Result<(), Box<dyn Error>> {
        let db = MemoryDb::new();

        db.accounts_create(Account {
            username: "me".into(),
            is_admin: false,
        })
        .await?;

        assert!(
            CheckWriteRightUseCase { db_service: &db }
                .run("me", GhUserPermission::Write)
                .await?
        );

        Ok(())
    }
}
