use github_scbot_core::crypto::{JwtUtils, RsaUtils};
use github_scbot_core::utils::TimeUtils;
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

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExternalAccount {
    pub username: String,
    pub public_key: String,
    pub private_key: String,
}

impl ExternalAccount {
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

    pub fn with_generated_keys(mut self) -> Self {
        let (private_key, public_key) = RsaUtils::generate_rsa_keys();
        self.private_key = private_key.to_string();
        self.public_key = public_key.to_string();
        self
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

#[cfg(test)]
mod new_tests {
    use crate::{utils::db_test_case, DatabaseError, ExternalAccount};

    #[actix_rt::test]
    async fn create_no_keys() {
        db_test_case("external_account_create_no_keys", |mut db| async move {
            let exa = db
                .external_accounts_create(ExternalAccount {
                    username: "me".into(),
                    ..Default::default()
                })
                .await?;
            assert_eq!(exa.username, "me");
            assert_eq!(exa.public_key, "");
            assert_eq!(exa.private_key, "");

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn create_keys() {
        db_test_case("external_account_create_keys", |mut db| async move {
            let exa = db
                .external_accounts_create(
                    ExternalAccount {
                        username: "me".into(),
                        ..Default::default()
                    }
                    .with_generated_keys(),
                )
                .await?;
            assert_eq!(exa.username, "me");
            assert_ne!(exa.public_key, "");
            assert_ne!(exa.private_key, "");

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn update() {
        db_test_case("external_account_update", |mut db| async move {
            assert!(matches!(
                db.external_accounts_update(ExternalAccount {
                    username: "me".into(),
                    ..Default::default()
                })
                .await,
                Err(DatabaseError::UnknownExternalAccount(_))
            ));

            db.external_accounts_create(ExternalAccount {
                username: "me".into(),
                ..Default::default()
            })
            .await?;
            let exa = db
                .external_accounts_update(
                    ExternalAccount {
                        username: "me".into(),
                        ..Default::default()
                    }
                    .with_generated_keys(),
                )
                .await?;
            assert_eq!(exa.username, "me");
            assert_ne!(exa.public_key, "");
            assert_ne!(exa.private_key, "");

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn set_keys() {
        db_test_case("external_account_set_keys", |mut db| async move {
            assert!(matches!(
                db.external_accounts_set_keys("me", "one", "two").await,
                Err(DatabaseError::UnknownExternalAccount(_))
            ));

            db.external_accounts_create(ExternalAccount {
                username: "me".into(),
                ..Default::default()
            })
            .await?;

            let exa = db.external_accounts_set_keys("me", "one", "two").await?;
            assert_eq!(exa.username, "me");
            assert_eq!(exa.public_key, "one");
            assert_eq!(exa.private_key, "two");

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn get() {
        db_test_case("external_account_get", |mut db| async move {
            assert_eq!(db.external_accounts_get("me").await?, None);

            let exa = db
                .external_accounts_create(ExternalAccount {
                    username: "me".into(),
                    ..Default::default()
                })
                .await?;

            let get_exa = db.external_accounts_get("me").await?;
            assert_eq!(Some(exa), get_exa);

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn delete() {
        db_test_case("external_account_delete", |mut db| async move {
            assert!(!db.external_accounts_delete("me").await?);

            db.external_accounts_create(ExternalAccount {
                username: "me".into(),
                ..Default::default()
            })
            .await?;

            assert!(db.external_accounts_delete("me").await?);

            let get_exa = db.external_accounts_get("me").await?;
            assert_eq!(get_exa, None);

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn all() {
        db_test_case("external_account_all", |mut db| async move {
            assert_eq!(db.external_accounts_all().await?, vec![]);

            let exa1 = db
                .external_accounts_create(ExternalAccount {
                    username: "me".into(),
                    ..Default::default()
                })
                .await?;
            let exa2 = db
                .external_accounts_create(ExternalAccount {
                    username: "him".into(),
                    ..Default::default()
                })
                .await?;
            let exa3 = db
                .external_accounts_create(ExternalAccount {
                    username: "her".into(),
                    ..Default::default()
                })
                .await?;

            assert_eq!(db.external_accounts_all().await?, vec![exa3, exa2, exa1]);

            Ok(())
        })
        .await;
    }
}
