use github_scbot_macros::SCGetter;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, FromRow, Row};

#[derive(
    SCGetter, Debug, Clone, Default, derive_builder::Builder, Serialize, Deserialize, PartialEq, Eq,
)]
#[builder(default, setter(into))]
pub struct Account {
    #[get_deref]
    pub(crate) username: String,
    #[get]
    pub(crate) is_admin: bool,
}

impl Account {
    pub fn builder() -> AccountBuilder {
        AccountBuilder::default()
    }
}

impl<'r> FromRow<'r, PgRow> for Account {
    fn from_row(row: &'r PgRow) -> core::result::Result<Self, sqlx::Error> {
        Ok(Self {
            username: row.try_get("username")?,
            is_admin: row.try_get("is_admin")?,
        })
    }
}

#[cfg(test)]
mod new_tests {
    use crate::{utils::db_test_case, Account};

    #[actix_rt::test]
    async fn default_account_is_not_admin() {
        db_test_case(
            "account_default_account_is_not_admin",
            |mut db| async move {
                let account = Account::builder().username("me").build()?;

                let account = db.accounts_create(account).await?;
                assert_eq!(account.username(), "me");
                assert!(!account.is_admin());

                Ok(())
            },
        )
        .await;
    }

    #[actix_rt::test]
    async fn create() {
        db_test_case("account_create", |mut db| async move {
            let account = Account::builder().username("me").is_admin(true).build()?;

            let account = db.accounts_create(account).await?;
            assert!(account.is_admin());

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn get() {
        db_test_case("account_get", |mut db| async move {
            let account = Account::builder().username("me").build()?;

            let account = db.accounts_create(account).await?;
            let get_account = db.accounts_get("me").await?;
            assert_eq!(Some(account), get_account);

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn delete() {
        db_test_case("account_delete", |mut db| async move {
            let account = Account::builder().username("me").build()?;

            db.accounts_create(account).await?;
            let found = db.accounts_delete("me").await?;
            assert!(found);
            assert_eq!(db.accounts_get("me").await?, None);

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn delete_not_found() {
        db_test_case("account_delete_not_found", |mut db| async move {
            let found = db.accounts_delete("me").await?;
            assert!(!found);

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn list_admins() {
        db_test_case("account_list_admins", |mut db| async move {
            let account1 = Account::builder().username("me").build()?;
            let account2 = Account::builder().username("him").is_admin(true).build()?;
            let account3 = Account::builder().username("her").is_admin(true).build()?;

            db.accounts_create(account1).await?;
            let account2 = db.accounts_create(account2).await?;
            let account3 = db.accounts_create(account3).await?;
            let admins = db.accounts_list_admins().await?;
            assert_eq!(admins, vec![account3, account2]);

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn all() {
        db_test_case("account_all", |mut db| async move {
            let account1 = Account::builder().username("me").build()?;
            let account2 = Account::builder().username("him").build()?;
            let account3 = Account::builder().username("her").build()?;

            let account1 = db.accounts_create(account1).await?;
            let account2 = db.accounts_create(account2).await?;
            let account3 = db.accounts_create(account3).await?;
            let accounts = db.accounts_all().await?;
            assert_eq!(accounts, vec![account3, account2, account1]);

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn is_admin() {
        db_test_case("account_set_is_admin", |mut db| async move {
            let account1 = Account::builder().username("me").build()?;

            db.accounts_create(account1).await?;
            let account1 = db.accounts_set_is_admin("me", true).await?;
            assert!(account1.is_admin());

            let account1 = db.accounts_set_is_admin("me", false).await?;
            assert!(!account1.is_admin());

            Ok(())
        })
        .await;
    }
}
