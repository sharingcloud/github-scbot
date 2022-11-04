use github_scbot_core::types::repository::RepositoryPath;
use github_scbot_database::{DbService, ExternalAccountRight};

use crate::Result;

#[async_trait::async_trait(?Send)]
pub trait AddAccountRightUseCaseInterface {
    async fn run(&self) -> Result<()>;
}

pub struct AddAccountRightUseCase<'a> {
    pub repository_path: RepositoryPath,
    pub username: String,
    pub db_service: &'a dyn DbService,
}

#[async_trait::async_trait(?Send)]
impl<'a> AddAccountRightUseCaseInterface for AddAccountRightUseCase<'a> {
    async fn run(&self) -> Result<()> {
        let (owner, name) = self.repository_path.components();
        let mut repo_db = self.db_service.repositories();
        let mut exr_db = self.db_service.external_account_rights();

        let repository = repo_db.get(owner, name).await?.unwrap();

        exr_db.delete(owner, name, &self.username).await?;
        exr_db
            .create(
                ExternalAccountRight::builder()
                    .repository_id(repository.id())
                    .username(self.username.clone())
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
    use github_scbot_database::{
        ExternalAccountRight, MockDbService, MockExternalAccountRightDB, MockRepositoryDB,
        Repository,
    };
    use mockall::predicate;

    use super::*;

    #[actix_rt::test]
    async fn test() -> Result<()> {
        let mut db_adapter = MockDbService::new();
        db_adapter.expect_repositories().returning(|| {
            let mut mock = MockRepositoryDB::new();
            mock.expect_get()
                .with(predicate::eq("owner"), predicate::eq("name"))
                .returning(|_, _| {
                    async { Ok(Some(Repository::builder().build().unwrap())) }.boxed()
                });

            Box::new(mock)
        });
        db_adapter.expect_external_account_rights().returning(|| {
            let mut mock = MockExternalAccountRightDB::new();
            mock.expect_delete()
                .with(
                    predicate::eq("owner"),
                    predicate::eq("name"),
                    predicate::eq("me"),
                )
                .returning(|_, _, _| async { Ok(false) }.boxed());
            mock.expect_create()
                .withf(|right| right.repository_id() == 0 && right.username() == "me")
                .returning(|_| {
                    async { Ok(ExternalAccountRight::builder().build().unwrap()) }.boxed()
                });

            Box::new(mock)
        });

        let use_case = AddAccountRightUseCase {
            username: "me".into(),
            repository_path: "owner/name".try_into().unwrap(),
            db_service: &db_adapter,
        };
        assert!(use_case.run().await.is_ok());

        Ok(())
    }
}
