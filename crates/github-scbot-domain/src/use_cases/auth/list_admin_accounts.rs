use github_scbot_database::{Account, DbService};

use crate::Result;

#[async_trait::async_trait(?Send)]
pub trait ListAdminAccountsUseCaseInterface {
    async fn run(&self) -> Result<Vec<Account>>;
}

pub struct ListAdminAccountsUseCase<'a> {
    pub db_service: &'a dyn DbService,
}

#[async_trait::async_trait(?Send)]
impl<'a> ListAdminAccountsUseCaseInterface for ListAdminAccountsUseCase<'a> {
    async fn run(&self) -> Result<Vec<Account>> {
        self.db_service
            .accounts()
            .list_admins()
            .await
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use futures_util::FutureExt;
    use github_scbot_database::{MockAccountDB, MockDbService};

    use super::*;

    #[actix_rt::test]
    async fn test_no_account() -> Result<()> {
        let mut db_adapter = MockDbService::new();
        db_adapter.expect_accounts().returning(|| {
            let mut mock = MockAccountDB::new();
            mock.expect_list_admins()
                .returning(|| async { Ok(vec![]) }.boxed());

            Box::new(mock)
        });

        let use_case = ListAdminAccountsUseCase {
            db_service: &db_adapter,
        };
        let result = use_case.run().await?;
        assert!(result.is_empty());

        Ok(())
    }

    #[actix_rt::test]
    async fn test_accounts() -> Result<()> {
        let mut db_adapter = MockDbService::new();
        db_adapter.expect_accounts().returning(|| {
            let mut mock = MockAccountDB::new();
            mock.expect_list_admins().returning(|| {
                async {
                    Ok(vec![
                        Account::builder().username("one").build().unwrap(),
                        Account::builder().username("two").build().unwrap(),
                    ])
                }
                .boxed()
            });

            Box::new(mock)
        });

        let use_case = ListAdminAccountsUseCase {
            db_service: &db_adapter,
        };
        let result = use_case.run().await?;
        assert_eq!(
            result,
            vec![
                Account::builder().username("one").build().unwrap(),
                Account::builder().username("two").build().unwrap(),
            ]
        );

        Ok(())
    }
}
