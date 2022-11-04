use github_scbot_database::DbService;

use crate::Result;

#[async_trait::async_trait(?Send)]
pub trait CreateExternalTokenUseCaseInterface {
    async fn run(&self) -> Result<String>;
}

pub struct CreateExternalTokenUseCase<'a> {
    pub username: String,
    pub db_service: &'a dyn DbService,
}

#[async_trait::async_trait(?Send)]
impl<'a> CreateExternalTokenUseCaseInterface for CreateExternalTokenUseCase<'a> {
    async fn run(&self) -> Result<String> {
        let mut exa_db = self.db_service.external_accounts();
        let exa = exa_db.get(&self.username).await?.unwrap();
        exa.generate_access_token().map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use futures_util::FutureExt;
    use github_scbot_database::{ExternalAccount, MockDbService, MockExternalAccountDB};
    use mockall::predicate;

    use super::*;

    #[actix_rt::test]
    async fn test() -> Result<()> {
        let mut db_adapter = MockDbService::new();
        db_adapter.expect_external_accounts().returning(|| {
            let mut mock = MockExternalAccountDB::new();
            mock.expect_get().with(predicate::eq("me")).returning(|_| {
                async {
                    Ok(Some(
                        ExternalAccount::builder().generate_keys().build().unwrap(),
                    ))
                }
                .boxed()
            });

            Box::new(mock)
        });

        let use_case = CreateExternalTokenUseCase {
            username: "me".into(),
            db_service: &db_adapter,
        };
        assert!(use_case.run().await.is_ok());

        Ok(())
    }
}
