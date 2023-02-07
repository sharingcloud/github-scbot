use github_scbot_database::{Account, DbServiceAll};

use crate::Result;

pub struct AddAdminRightUseCase<'a> {
    pub username: &'a str,
    pub db_service: &'a mut dyn DbServiceAll,
}

impl<'a> AddAdminRightUseCase<'a> {
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

    use github_scbot_database::{DbServiceAll, MemoryDb};

    use super::AddAdminRightUseCase;

    #[actix_rt::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let mut db = MemoryDb::new();

        AddAdminRightUseCase {
            username: "me",
            db_service: &mut db,
        }
        .run()
        .await?;

        assert!(db.accounts_get_expect("me").await?.is_admin);

        Ok(())
    }
}
