use github_scbot_macros::SCGetter;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, FromRow, Row};

use crate::Repository;

#[derive(
    SCGetter, Debug, Clone, Default, derive_builder::Builder, Serialize, Deserialize, PartialEq, Eq,
)]
#[builder(default, setter(into))]
pub struct ExternalAccountRight {
    #[get_deref]
    pub(crate) username: String,
    #[get]
    pub(crate) repository_id: u64,
}

impl ExternalAccountRight {
    pub fn builder() -> ExternalAccountRightBuilder {
        ExternalAccountRightBuilder::default()
    }

    pub fn set_repository_id(&mut self, id: u64) {
        self.repository_id = id;
    }
}

impl ExternalAccountRightBuilder {
    pub fn with_repository(&mut self, repository: &Repository) -> &mut Self {
        self.repository_id = Some(repository.id());
        self
    }
}

impl<'r> FromRow<'r, PgRow> for ExternalAccountRight {
    fn from_row(row: &'r PgRow) -> core::result::Result<Self, sqlx::Error> {
        Ok(Self {
            username: row.try_get("username")?,
            repository_id: row.try_get::<i32, _>("repository_id")? as u64,
        })
    }
}

#[cfg(test)]
mod new_tests {
    use crate::{utils::db_test_case, ExternalAccount, ExternalAccountRight, Repository};

    #[actix_rt::test]
    async fn create() {
        db_test_case("external_account_right_create", |mut db| async move {
            let repo = db
                .repositories_create(Repository::builder().owner("me").name("repo").build()?)
                .await?;
            let exa = db
                .external_accounts_create(ExternalAccount::builder().username("me").build()?)
                .await?;
            let exr = db
                .external_account_rights_create(
                    ExternalAccountRight::builder()
                        .repository_id(repo.id())
                        .username(exa.username())
                        .build()?,
                )
                .await?;

            assert_eq!(exr.username(), exa.username());
            assert_eq!(exr.repository_id(), repo.id());

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn get() {
        db_test_case("external_account_right_get", |mut db| async move {
            let repo = db
                .repositories_create(Repository::builder().owner("me").name("repo").build()?)
                .await?;
            let exa = db
                .external_accounts_create(ExternalAccount::builder().username("me").build()?)
                .await?;

            assert_eq!(
                db.external_account_rights_get("me", "repo", "me").await?,
                None
            );

            let exr = db
                .external_account_rights_create(
                    ExternalAccountRight::builder()
                        .repository_id(repo.id())
                        .username(exa.username())
                        .build()?,
                )
                .await?;

            let get_exr = db.external_account_rights_get("me", "repo", "me").await?;
            assert_eq!(Some(exr), get_exr);

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn delete() {
        db_test_case("external_account_right_delete", |mut db| async move {
            let repo = db
                .repositories_create(Repository::builder().owner("me").name("repo").build()?)
                .await?;
            let exa = db
                .external_accounts_create(ExternalAccount::builder().username("me").build()?)
                .await?;

            assert_eq!(
                db.external_account_rights_delete("me", "repo", "me")
                    .await?,
                false
            );

            db.external_account_rights_create(
                ExternalAccountRight::builder()
                    .repository_id(repo.id())
                    .username(exa.username())
                    .build()?,
            )
            .await?;

            assert_eq!(
                db.external_account_rights_delete("me", "repo", "me")
                    .await?,
                true
            );
            assert_eq!(
                db.external_account_rights_get("me", "repo", "me").await?,
                None
            );

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn delete_all() {
        db_test_case("external_account_right_delete_all", |mut db| async move {
            let repo1 = db
                .repositories_create(Repository::builder().owner("me").name("repo1").build()?)
                .await?;
            let repo2 = db
                .repositories_create(Repository::builder().owner("me").name("repo2").build()?)
                .await?;
            let exa1 = db
                .external_accounts_create(ExternalAccount::builder().username("me").build()?)
                .await?;
            let exa2 = db
                .external_accounts_create(ExternalAccount::builder().username("me2").build()?)
                .await?;

            assert_eq!(db.external_account_rights_delete_all("me").await?, false);

            db.external_account_rights_create(
                ExternalAccountRight::builder()
                    .repository_id(repo1.id())
                    .username(exa1.username())
                    .build()?,
            )
            .await?;
            db.external_account_rights_create(
                ExternalAccountRight::builder()
                    .repository_id(repo2.id())
                    .username(exa1.username())
                    .build()?,
            )
            .await?;
            let exr3 = db
                .external_account_rights_create(
                    ExternalAccountRight::builder()
                        .repository_id(repo2.id())
                        .username(exa2.username())
                        .build()?,
                )
                .await?;

            assert_eq!(db.external_account_rights_delete_all("me").await?, true);
            assert_eq!(db.external_account_rights_list("me").await?, vec![]);
            assert_eq!(db.external_account_rights_list("me2").await?, vec![exr3]);

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn list() {
        db_test_case("external_account_right_list", |mut db| async move {
            let repo1 = db
                .repositories_create(Repository::builder().owner("me").name("repo1").build()?)
                .await?;
            let repo2 = db
                .repositories_create(Repository::builder().owner("me").name("repo2").build()?)
                .await?;
            let exa1 = db
                .external_accounts_create(ExternalAccount::builder().username("me").build()?)
                .await?;

            let exr1 = db
                .external_account_rights_create(
                    ExternalAccountRight::builder()
                        .repository_id(repo1.id())
                        .username(exa1.username())
                        .build()?,
                )
                .await?;
            let exr2 = db
                .external_account_rights_create(
                    ExternalAccountRight::builder()
                        .repository_id(repo2.id())
                        .username(exa1.username())
                        .build()?,
                )
                .await?;

            assert_eq!(
                db.external_account_rights_list("me").await?,
                vec![exr1, exr2]
            );

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn all() {
        db_test_case("external_account_right_all", |mut db| async move {
            let repo1 = db
                .repositories_create(Repository::builder().owner("me").name("repo1").build()?)
                .await?;
            let repo2 = db
                .repositories_create(Repository::builder().owner("me").name("repo2").build()?)
                .await?;
            let exa1 = db
                .external_accounts_create(ExternalAccount::builder().username("me").build()?)
                .await?;
            let exa2 = db
                .external_accounts_create(ExternalAccount::builder().username("her").build()?)
                .await?;

            let exr1 = db
                .external_account_rights_create(
                    ExternalAccountRight::builder()
                        .repository_id(repo1.id())
                        .username(exa1.username())
                        .build()?,
                )
                .await?;
            let exr2 = db
                .external_account_rights_create(
                    ExternalAccountRight::builder()
                        .repository_id(repo2.id())
                        .username(exa1.username())
                        .build()?,
                )
                .await?;
            let exr3 = db
                .external_account_rights_create(
                    ExternalAccountRight::builder()
                        .repository_id(repo1.id())
                        .username(exa2.username())
                        .build()?,
                )
                .await?;
            let exr4 = db
                .external_account_rights_create(
                    ExternalAccountRight::builder()
                        .repository_id(repo2.id())
                        .username(exa2.username())
                        .build()?,
                )
                .await?;

            assert_eq!(
                db.external_account_rights_all().await?,
                vec![exr3, exr4, exr1, exr2]
            );

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn cascade_external_account() {
        db_test_case(
            "external_account_right_cascade_external_account",
            |mut db| async move {
                let repo = db
                    .repositories_create(Repository::builder().owner("me").name("repo").build()?)
                    .await?;
                let exa = db
                    .external_accounts_create(ExternalAccount::builder().username("me").build()?)
                    .await?;
                db.external_account_rights_create(
                    ExternalAccountRight::builder()
                        .repository_id(repo.id())
                        .username(exa.username())
                        .build()?,
                )
                .await?;

                // On account deletion, rights should be dropped
                db.external_accounts_delete("me").await?;
                assert_eq!(db.external_account_rights_all().await?, vec![]);

                Ok(())
            },
        )
        .await
    }

    #[actix_rt::test]
    async fn cascade_repository() {
        db_test_case(
            "external_account_right_cascade_repository",
            |mut db| async move {
                let repo = db
                    .repositories_create(Repository::builder().owner("me").name("repo").build()?)
                    .await?;
                let exa = db
                    .external_accounts_create(ExternalAccount::builder().username("me").build()?)
                    .await?;
                db.external_account_rights_create(
                    ExternalAccountRight::builder()
                        .repository_id(repo.id())
                        .username(exa.username())
                        .build()?,
                )
                .await?;

                // On repository deletion, rights should be dropped
                db.repositories_delete("me", "repo").await?;
                assert_eq!(db.external_account_rights_list("me").await?, vec![]);

                Ok(())
            },
        )
        .await
    }
}
