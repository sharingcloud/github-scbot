use github_scbot_crypto::JwtUtils;
use github_scbot_utils::TimeUtils;
use serde::{Deserialize, Serialize};

use crate::{schema::external_account, Result};

mod adapter;
mod builder;
pub use adapter::{
    DummyExternalAccountDbAdapter, ExternalAccountDbAdapter, IExternalAccountDbAdapter,
};
use builder::ExternalAccountModelBuilder;

/// External Jwt claims.
#[derive(Debug, Serialize, Deserialize)]
pub struct ExternalJwtClaims {
    /// Issued at time
    pub iat: u64,
    /// Identifier
    pub iss: String,
}

/// External account.
#[derive(
    Debug,
    Deserialize,
    Insertable,
    Identifiable,
    Serialize,
    Queryable,
    Clone,
    Default,
    AsChangeset,
    PartialEq,
    Eq,
)]
#[primary_key(username)]
#[table_name = "external_account"]
pub struct ExternalAccountModel {
    /// Username.
    pub username: String,
    /// Public key.
    pub public_key: String,
    /// Private key.
    pub private_key: String,
}

impl ExternalAccountModel {
    /// Create builder.
    pub fn builder(username: &str) -> ExternalAccountModelBuilder {
        ExternalAccountModelBuilder::default(username)
    }

    /// Create builder from model.
    pub fn builder_from_model(model: &Self) -> ExternalAccountModelBuilder {
        ExternalAccountModelBuilder::from_model(model)
    }

    /// Generate access token.
    pub fn generate_access_token(&self) -> Result<String> {
        let now_ts = TimeUtils::now_timestamp();
        let claims = ExternalJwtClaims {
            // Issued at time
            iat: now_ts,
            // Username
            iss: self.username.clone(),
        };

        JwtUtils::create_jwt(&self.private_key, &claims).map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{tests::using_test_db, DatabaseError};

    #[actix_rt::test]
    async fn create_and_update() -> Result<()> {
        using_test_db("test_db_external_account", |_config, pool| async move {
            let db_adapter = ExternalAccountDbAdapter::new(pool.clone());
            let acc = ExternalAccountModel::builder("ext1")
                .create_or_update(&db_adapter)
                .await
                .unwrap();

            assert_eq!(
                acc,
                ExternalAccountModel {
                    username: "ext1".into(),
                    public_key: String::new(),
                    private_key: String::new(),
                }
            );

            let acc = ExternalAccountModel::builder("ext1")
                .private_key("pri")
                .public_key("pub")
                .create_or_update(&db_adapter)
                .await
                .unwrap();

            assert_eq!(
                acc,
                ExternalAccountModel {
                    username: "ext1".into(),
                    private_key: "pri".into(),
                    public_key: "pub".into()
                }
            );

            let acc = ExternalAccountModel::builder("ext1")
                .public_key("public")
                .create_or_update(&db_adapter)
                .await
                .unwrap();

            assert_eq!(
                acc,
                ExternalAccountModel {
                    username: "ext1".into(),
                    private_key: "pri".into(),
                    public_key: "public".into()
                }
            );

            // Only one account after 3 create_or_update.
            assert_eq!(db_adapter.list().await.unwrap().len(), 1);
            Ok::<_, DatabaseError>(())
        })
        .await
    }
}
