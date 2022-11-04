use github_scbot_database::{DbService, Repository};

use crate::Result;

#[async_trait::async_trait(?Send)]
pub trait ListAccountRightsUseCaseInterface {
    async fn run(&self) -> Result<Vec<Repository>>;
}

pub struct ListAccountRightsUseCase<'a> {
    pub username: String,
    pub db_service: &'a dyn DbService,
}

#[async_trait::async_trait(?Send)]
impl<'a> ListAccountRightsUseCaseInterface for ListAccountRightsUseCase<'a> {
    async fn run(&self) -> Result<Vec<Repository>> {
        let mut exr_db = self.db_service.external_account_rights();
        let rights = exr_db.list(&self.username).await?;

        let mut repositories = Vec::new();
        if !rights.is_empty() {
            let mut repo_db = self.db_service.repositories();
            for right in rights {
                repositories.push(repo_db.get_from_id(right.repository_id()).await?.unwrap());
            }
        }

        Ok(repositories)
    }
}

#[cfg(test)]
mod tests {
    use futures_util::FutureExt;
    use github_scbot_database::{
        ExternalAccount, ExternalAccountRight, MockDbService, MockExternalAccountDB,
        MockExternalAccountRightDB, MockRepositoryDB,
    };
    use mockall::predicate;

    use super::*;

    #[actix_rt::test]
    async fn test_no_right() -> Result<()> {
        let mut db_adapter = MockDbService::new();
        db_adapter.expect_external_account_rights().returning(|| {
            let mut mock = MockExternalAccountRightDB::new();
            mock.expect_list()
                .with(predicate::eq("me"))
                .returning(|_| async { Ok(vec![]) }.boxed());

            Box::new(mock)
        });

        db_adapter.expect_external_accounts().returning(|| {
            let mut mock = MockExternalAccountDB::new();
            mock.expect_get().with(predicate::eq("me")).returning(|_| {
                async { Ok(Some(ExternalAccount::builder().build().unwrap())) }.boxed()
            });

            Box::new(mock)
        });

        let use_case = ListAccountRightsUseCase {
            username: "me".into(),
            db_service: &db_adapter,
        };
        let result = use_case.run().await?;
        assert!(result.is_empty());

        Ok(())
    }

    #[actix_rt::test]
    async fn test_rights() -> Result<()> {
        let mut db_adapter = MockDbService::new();
        db_adapter.expect_repositories().returning(|| {
            let mut mock = MockRepositoryDB::new();
            mock.expect_get_from_id()
                .with(predicate::eq(1))
                .returning(|_| {
                    async {
                        Ok(Some(
                            Repository::builder()
                                .owner("owner")
                                .name("name")
                                .build()
                                .unwrap(),
                        ))
                    }
                    .boxed()
                });
            mock.expect_get_from_id()
                .with(predicate::eq(2))
                .returning(|_| {
                    async {
                        Ok(Some(
                            Repository::builder()
                                .owner("owner")
                                .name("name2")
                                .build()
                                .unwrap(),
                        ))
                    }
                    .boxed()
                });

            Box::new(mock)
        });

        db_adapter.expect_external_account_rights().returning(|| {
            let mut mock = MockExternalAccountRightDB::new();
            mock.expect_list().with(predicate::eq("me")).returning(|_| {
                async {
                    Ok(vec![
                        ExternalAccountRight::builder()
                            .repository_id(1u64)
                            .build()
                            .unwrap(),
                        ExternalAccountRight::builder()
                            .repository_id(2u64)
                            .build()
                            .unwrap(),
                    ])
                }
                .boxed()
            });

            Box::new(mock)
        });

        db_adapter.expect_external_accounts().returning(|| {
            let mut mock = MockExternalAccountDB::new();
            mock.expect_get().with(predicate::eq("me")).returning(|_| {
                async { Ok(Some(ExternalAccount::builder().build().unwrap())) }.boxed()
            });

            Box::new(mock)
        });

        let use_case = ListAccountRightsUseCase {
            username: "me".into(),
            db_service: &db_adapter,
        };
        assert_eq!(
            use_case.run().await?,
            vec![
                Repository::builder()
                    .owner("owner")
                    .name("name")
                    .build()
                    .unwrap(),
                Repository::builder()
                    .owner("owner")
                    .name("name2")
                    .build()
                    .unwrap(),
            ]
        );

        Ok(())
    }
}
