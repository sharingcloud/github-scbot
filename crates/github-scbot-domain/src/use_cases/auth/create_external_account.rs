use github_scbot_database::{DbService, ExternalAccount};

use crate::Result;

#[async_trait::async_trait(?Send)]
pub trait CreateExternalAccountUseCaseInterface {
    async fn run(&self) -> Result<()>;
}

pub struct CreateExternalAccountUseCase<'a> {
    pub username: String,
    pub db_service: &'a dyn DbService,
}

#[async_trait::async_trait(?Send)]
impl<'a> CreateExternalAccountUseCaseInterface for CreateExternalAccountUseCase<'a> {
    async fn run(&self) -> Result<()> {
        let mut exa_db = self.db_service.external_accounts();
        exa_db
            .create(
                ExternalAccount::builder()
                    .username(self.username.clone())
                    .generate_keys()
                    .build()
                    .unwrap(),
            )
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use futures_util::FutureExt;
    use github_scbot_database::{MockDbService, MockExternalAccountDB};
    use mockall::predicate;

    use super::*;

    #[actix_rt::test]
    async fn test() -> Result<()> {
        let mut db_adapter = MockDbService::new();
        db_adapter.expect_external_accounts().returning(|| {
            let mut mock = MockExternalAccountDB::new();
            mock.expect_get()
                .with(predicate::eq("me"))
                .returning(|_| async { Ok(None) }.boxed());

            mock.expect_create()
                .withf(|acc| acc.username() == "me")
                .returning(|_| async { Ok(ExternalAccount::builder().build().unwrap()) }.boxed());

            Box::new(mock)
        });

        let use_case = CreateExternalAccountUseCase {
            username: "me".into(),
            db_service: &db_adapter,
        };
        assert!(use_case.run().await.is_ok());

        Ok(())
    }
}
