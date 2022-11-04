use github_scbot_database::{DbService, ExternalAccount};

use crate::Result;

#[async_trait::async_trait(?Send)]
pub trait ListExternalAccountsUseCaseInterface {
    async fn run(&self) -> Result<Vec<ExternalAccount>>;
}

pub struct ListExternalAccountsUseCase<'a> {
    pub db_service: &'a dyn DbService,
}

#[async_trait::async_trait(?Send)]
impl<'a> ListExternalAccountsUseCaseInterface for ListExternalAccountsUseCase<'a> {
    async fn run(&self) -> Result<Vec<ExternalAccount>> {
        self.db_service
            .external_accounts()
            .all()
            .await
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use futures_util::FutureExt;
    use github_scbot_database::{MockDbService, MockExternalAccountDB};

    use super::*;

    #[actix_rt::test]
    async fn test_no_account() -> Result<()> {
        let mut db_adapter = MockDbService::new();
        db_adapter.expect_external_accounts().returning(|| {
            let mut mock = MockExternalAccountDB::new();
            mock.expect_all().returning(|| async { Ok(vec![]) }.boxed());

            Box::new(mock)
        });

        let use_case = ListExternalAccountsUseCase {
            db_service: &db_adapter,
        };
        let result = use_case.run().await?;
        assert!(result.is_empty());

        Ok(())
    }

    #[actix_rt::test]
    async fn test_accounts() -> Result<()> {
        let mut db_adapter = MockDbService::new();
        db_adapter.expect_external_accounts().returning(|| {
            let mut mock = MockExternalAccountDB::new();
            mock.expect_all().returning(|| {
                async {
                    Ok(vec![
                        ExternalAccount::builder().username("one").build().unwrap(),
                        ExternalAccount::builder().username("two").build().unwrap(),
                    ])
                }
                .boxed()
            });

            Box::new(mock)
        });

        let use_case = ListExternalAccountsUseCase {
            db_service: &db_adapter,
        };
        let result = use_case.run().await?;
        assert_eq!(
            result,
            vec![
                ExternalAccount::builder().username("one").build().unwrap(),
                ExternalAccount::builder().username("two").build().unwrap(),
            ]
        );

        Ok(())
    }
}
