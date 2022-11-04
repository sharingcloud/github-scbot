use github_scbot_core::types::repository::RepositoryPath;
use github_scbot_database::DbService;

use crate::Result;

#[async_trait::async_trait(?Send)]
pub trait RemoveAccountRightUseCaseInterface {
    async fn run(&self) -> Result<()>;
}

pub struct RemoveAccountRightUseCase<'a> {
    pub username: String,
    pub repository_path: RepositoryPath,
    pub db_service: &'a dyn DbService,
}

#[async_trait::async_trait(?Send)]
impl<'a> RemoveAccountRightUseCaseInterface for RemoveAccountRightUseCase<'a> {
    async fn run(&self) -> Result<()> {
        let (owner, name) = self.repository_path.components();
        let mut exr_db = self.db_service.external_account_rights();

        exr_db.delete(owner, name, &self.username).await?;

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
            mock.expect_delete()
                .with(
                    predicate::eq("owner"),
                    predicate::eq("name"),
                    predicate::eq("me"),
                )
                .returning(|_, _, _| async { Ok(true) }.boxed());

            Box::new(mock)
        });

        let use_case = RemoveAccountRightUseCase {
            username: "me".into(),
            repository_path: "owner/name".try_into().unwrap(),
            db_service: &db_adapter,
        };
        use_case.run().await?;

        Ok(())
    }
}
