use github_scbot_database::DbService;

use crate::Result;

#[async_trait::async_trait(?Send)]
pub trait RemoveExternalAccountUseCaseInterface {
    async fn run(&self) -> Result<()>;
}

pub struct RemoveExternalAccountUseCase<'a> {
    pub username: String,
    pub db_service: &'a dyn DbService,
}

#[async_trait::async_trait(?Send)]
impl<'a> RemoveExternalAccountUseCaseInterface for RemoveExternalAccountUseCase<'a> {
    async fn run(&self) -> Result<()> {
        let mut exa_db = self.db_service.external_accounts();
        exa_db.delete(&self.username).await?;

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
            mock.expect_delete()
                .with(predicate::eq("me"))
                .returning(|_| async { Ok(false) }.boxed());

            Box::new(mock)
        });

        let use_case = RemoveExternalAccountUseCase {
            username: "me".into(),
            db_service: &db_adapter,
        };
        assert!(use_case.run().await.is_ok());

        Ok(())
    }
}
