use github_scbot_core::crypto::{JwtUtils, RsaUtils};
use github_scbot_core::utils::TimeUtils;
use github_scbot_macros::SCGetter;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, FromRow, Row};

use crate::{DatabaseError, Result};

/// External Jwt claims.
#[derive(Debug, Serialize, Deserialize)]
pub struct ExternalJwtClaims {
    /// Issued at time
    pub iat: u64,
    /// Identifier
    pub iss: String,
}

#[derive(
    SCGetter, Debug, Clone, Default, derive_builder::Builder, Serialize, Deserialize, PartialEq, Eq,
)]
#[builder(default, setter(into))]
pub struct ExternalAccount {
    #[get_deref]
    pub(crate) username: String,
    #[get_deref]
    pub(crate) public_key: String,
    #[get_deref]
    pub(crate) private_key: String,
}

impl ExternalAccount {
    pub fn builder() -> ExternalAccountBuilder {
        ExternalAccountBuilder::default()
    }

    pub fn generate_access_token(&self) -> Result<String> {
        let now_ts = TimeUtils::now_timestamp();
        let claims = ExternalJwtClaims {
            // Issued at time
            iat: now_ts,
            // Username
            iss: self.username.clone(),
        };

        JwtUtils::create_jwt(&self.private_key, &claims)
            .map_err(|e| DatabaseError::CryptoError { source: e })
    }
}

impl<'r> FromRow<'r, PgRow> for ExternalAccount {
    fn from_row(row: &'r PgRow) -> core::result::Result<Self, sqlx::Error> {
        Ok(Self {
            username: row.try_get("username")?,
            public_key: row.try_get("public_key")?,
            private_key: row.try_get("private_key")?,
        })
    }
}

impl ExternalAccountBuilder {
    pub fn generate_keys(&mut self) -> &mut Self {
        let (private_key, public_key) = RsaUtils::generate_rsa_keys();
        self.private_key = Some(private_key.to_string());
        self.public_key = Some(public_key.to_string());
        self
    }
}

#[cfg(test)]
mod new_tests {
    use crate::{utils::db_test_case, ExternalAccount};

    #[actix_rt::test]
    async fn create_no_keys() {
        db_test_case("external_account_create_no_keys", |mut db| async move {
            let exa = ExternalAccount::builder().username("me").build()?;
            let exa = db.external_accounts_create(exa).await?;
            assert_eq!(exa.username(), "me");
            assert_eq!(exa.public_key(), "");
            assert_eq!(exa.private_key(), "");

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn create_keys() {
        db_test_case("external_account_create_keys", |mut db| async move {
            let exa = ExternalAccount::builder()
                .username("me")
                .generate_keys()
                .build()?;
            let exa = db.external_accounts_create(exa).await?;
            assert_eq!(exa.username(), "me");
            assert_ne!(exa.public_key(), "");
            assert_ne!(exa.private_key(), "");

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn set_keys() {
        db_test_case("external_account_set_keys", |mut db| async move {
            let exa = ExternalAccount::builder().username("me").build()?;
            db.external_accounts_create(exa).await?;

            let exa = db.external_accounts_set_keys("me", "one", "two").await?;
            assert_eq!(exa.username(), "me");
            assert_eq!(exa.public_key(), "one");
            assert_eq!(exa.private_key(), "two");

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn get() {
        db_test_case("external_account_get", |mut db| async move {
            let exa = ExternalAccount::builder()
                .username("me")
                .generate_keys()
                .build()?;
            let exa = db.external_accounts_create(exa).await?;
            let get_exa = db.external_accounts_get("me").await?;
            assert_eq!(Some(exa), get_exa);

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn delete() {
        db_test_case("external_account_delete", |mut db| async move {
            let exa = ExternalAccount::builder()
                .username("me")
                .generate_keys()
                .build()?;
            db.external_accounts_create(exa).await?;
            let found = db.external_accounts_delete("me").await?;
            assert!(found);

            let get_exa = db.external_accounts_get("me").await?;
            assert_eq!(get_exa, None);

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn delete_not_found() {
        db_test_case("external_account_delete_not_found", |mut db| async move {
            let found = db.external_accounts_delete("me").await?;
            assert!(!found);

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn all() {
        db_test_case("external_account_all", |mut db| async move {
            assert_eq!(db.external_accounts_all().await?, vec![]);

            let exa1 = ExternalAccount::builder().username("me").build()?;
            let exa2 = ExternalAccount::builder().username("him").build()?;
            let exa3 = ExternalAccount::builder().username("her").build()?;

            let exa1 = db.external_accounts_create(exa1).await?;
            let exa2 = db.external_accounts_create(exa2).await?;
            let exa3 = db.external_accounts_create(exa3).await?;
            assert_eq!(db.external_accounts_all().await?, vec![exa3, exa2, exa1]);

            Ok(())
        })
        .await;
    }
}
