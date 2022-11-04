use github_scbot_database::{Account, DbService};

use crate::Result;

#[async_trait::async_trait(?Send)]
pub trait RemoveAdminRightUseCaseInterface {
    async fn run(&self) -> Result<()>;
}

pub struct RemoveAdminRightUseCase<'a> {
    pub username: String,
    pub db_service: &'a dyn DbService,
}

#[async_trait::async_trait(?Send)]
impl<'a> RemoveAdminRightUseCaseInterface for RemoveAdminRightUseCase<'a> {
    async fn run(&self) -> Result<()> {
        let mut acc_db = self.db_service.accounts();
        match acc_db.get(&self.username).await? {
            Some(_) => acc_db.set_is_admin(&self.username, false).await?,
            None => {
                acc_db
                    .create(
                        Account::builder()
                            .username(self.username.clone())
                            .is_admin(false)
                            .build()
                            .unwrap(),
                    )
                    .await?
            }
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use futures_util::FutureExt;
    use github_scbot_database::{MockAccountDB, MockDbService};
    use mockall::predicate;

    use super::*;

    #[actix_rt::test]
    async fn test_existing() -> Result<()> {
        let mut db_adapter = MockDbService::new();
        db_adapter.expect_accounts().returning(|| {
            let mut mock = MockAccountDB::new();
            mock.expect_get()
                .with(predicate::eq("me"))
                .returning(|_| async { Ok(Some(Account::builder().build().unwrap())) }.boxed());
            mock.expect_set_is_admin()
                .with(predicate::eq("me"), predicate::eq(false))
                .returning(|_, _| async { Ok(Account::builder().build().unwrap()) }.boxed());

            Box::new(mock)
        });

        let use_case = RemoveAdminRightUseCase {
            username: "me".into(),
            db_service: &db_adapter,
        };
        assert!(use_case.run().await.is_ok());

        Ok(())
    }

    #[actix_rt::test]
    async fn test_not_existing() -> Result<()> {
        let mut db_adapter = MockDbService::new();
        db_adapter.expect_accounts().returning(|| {
            let mut mock = MockAccountDB::new();
            mock.expect_get()
                .with(predicate::eq("me"))
                .returning(|_| async { Ok(None) }.boxed());

            mock.expect_create()
                .withf(|acc| acc.username() == "me" && !acc.is_admin())
                .returning(|_| async { Ok(Account::builder().build().unwrap()) }.boxed());

            Box::new(mock)
        });

        let use_case = RemoveAdminRightUseCase {
            username: "me".into(),
            db_service: &db_adapter,
        };
        assert!(use_case.run().await.is_ok());

        Ok(())
    }
}
