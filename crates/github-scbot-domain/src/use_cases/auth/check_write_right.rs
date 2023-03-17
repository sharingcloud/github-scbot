use github_scbot_core::types::common::GhUserPermission;
use github_scbot_database_interface::DbService;

use super::check_is_admin::CheckIsAdminUseCase;
use crate::Result;

pub struct CheckWriteRightUseCase<'a> {
    pub username: &'a str,
    pub user_permission: GhUserPermission,
    pub db_service: &'a mut dyn DbService,
}

impl<'a> CheckWriteRightUseCase<'a> {
    pub async fn run(&mut self) -> Result<bool> {
        let is_admin = CheckIsAdminUseCase {
            username: self.username,
            db_service: self.db_service,
        }
        .run()
        .await?;

        Ok(is_admin || self.user_permission.can_write())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use github_scbot_core::types::common::GhUserPermission;
    use github_scbot_database_interface::DbService;
    use github_scbot_database_memory::MemoryDb;
    use github_scbot_domain_models::Account;

    use super::CheckWriteRightUseCase;

    #[actix_rt::test]
    async fn run_read_not_admin() -> Result<(), Box<dyn Error>> {
        let mut db = MemoryDb::new();

        db.accounts_create(Account {
            username: "me".into(),
            is_admin: false,
        })
        .await?;

        assert!(
            !CheckWriteRightUseCase {
                username: "me",
                user_permission: GhUserPermission::Read,
                db_service: &mut db,
            }
            .run()
            .await?
        );

        Ok(())
    }

    #[actix_rt::test]
    async fn run_read_admin() -> Result<(), Box<dyn Error>> {
        let mut db = MemoryDb::new();

        db.accounts_create(Account {
            username: "me".into(),
            is_admin: true,
        })
        .await?;

        assert!(
            CheckWriteRightUseCase {
                username: "me",
                user_permission: GhUserPermission::Read,
                db_service: &mut db,
            }
            .run()
            .await?
        );

        Ok(())
    }

    #[actix_rt::test]
    async fn run_write_admin() -> Result<(), Box<dyn Error>> {
        let mut db = MemoryDb::new();

        db.accounts_create(Account {
            username: "me".into(),
            is_admin: true,
        })
        .await?;

        assert!(
            CheckWriteRightUseCase {
                username: "me",
                user_permission: GhUserPermission::Write,
                db_service: &mut db,
            }
            .run()
            .await?
        );

        Ok(())
    }

    #[actix_rt::test]
    async fn run_write_not_admin() -> Result<(), Box<dyn Error>> {
        let mut db = MemoryDb::new();

        db.accounts_create(Account {
            username: "me".into(),
            is_admin: false,
        })
        .await?;

        assert!(
            CheckWriteRightUseCase {
                username: "me",
                user_permission: GhUserPermission::Write,
                db_service: &mut db,
            }
            .run()
            .await?
        );

        Ok(())
    }
}
