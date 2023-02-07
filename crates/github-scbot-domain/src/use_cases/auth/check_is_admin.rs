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

#[cfg(test)]
mod tests {
    use std::error::Error;

    use github_scbot_database::{Account, DbServiceAll, MemoryDb};

    use super::CheckIsAdminUseCase;

    #[actix_rt::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let mut db = MemoryDb::new();

        db.accounts_create(Account {
            username: "me".into(),
            is_admin: true,
        })
        .await?;

        assert!(
            CheckIsAdminUseCase {
                username: "me",
                db_service: &mut db,
            }
            .run()
            .await?
        );

        Ok(())
    }

    #[actix_rt::test]
    async fn run_not_admin() -> Result<(), Box<dyn Error>> {
        let mut db = MemoryDb::new();

        db.accounts_create(Account {
            username: "me".into(),
            is_admin: false,
        })
        .await?;

        assert!(
            !CheckIsAdminUseCase {
                username: "me",
                db_service: &mut db,
            }
            .run()
            .await?
        );

        Ok(())
    }
}
