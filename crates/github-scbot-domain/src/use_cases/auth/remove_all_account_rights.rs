use github_scbot_database::DbService;

use crate::Result;

#[async_trait::async_trait(?Send)]
pub trait RemoveAllAccountRightsUseCaseInterface {
    async fn run(&self) -> Result<()>;
}

pub struct RemoveAllAccountRightsUseCase<'a> {
    pub username: String,
    pub db_service: &'a dyn DbService,
}

#[async_trait::async_trait(?Send)]
impl<'a> RemoveAllAccountRightsUseCaseInterface for RemoveAllAccountRightsUseCase<'a> {
    async fn run(&self) -> Result<()> {
        let mut exr_db = self.db_service.external_account_rights();
        exr_db.delete_all(&self.username).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use futures_util::FutureExt;
    use github_scbot_database::{MockDbService, MockExternalAccountRightDB};
    use mockall::predicate;

    use super::*;

    #[actix_rt::test]
    async fn test_no_account() -> Result<()> {
        let mut db_adapter = MockDbService::new();
        db_adapter.expect_external_account_rights().returning(|| {
            let mut mock = MockExternalAccountRightDB::new();
            mock.expect_delete_all()
                .with(predicate::eq("me"))
                .returning(|_| async { Ok(true) }.boxed());

            Box::new(mock)
        });

        let use_case = RemoveAllAccountRightsUseCase {
            username: "me".into(),
            db_service: &db_adapter,
        };
        use_case.run().await?;

        Ok(())
    }
}
