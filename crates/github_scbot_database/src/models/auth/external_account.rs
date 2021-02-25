use diesel::prelude::*;
use github_scbot_crypto::{create_jwt, generate_rsa_keys, now};
use serde::{Deserialize, Serialize};

use crate::{schema::external_account, DatabaseError, DbConn, Result};

/// External JWT claims.
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

#[must_use]
pub struct ExternalAccountModelBuilder {
    username: String,
    public_key: Option<String>,
    private_key: Option<String>,
}

impl ExternalAccountModelBuilder {
    pub fn default<T: Into<String>>(username: T) -> Self {
        Self {
            username: username.into(),
            public_key: None,
            private_key: None,
        }
    }

    pub fn from_model(model: &ExternalAccountModel) -> Self {
        Self {
            username: model.username.clone(),
            private_key: Some(model.private_key.clone()),
            public_key: Some(model.public_key.clone()),
        }
    }

    pub fn private_key<T: Into<String>>(mut self, key: T) -> Self {
        self.private_key = Some(key.into());
        self
    }

    pub fn public_key<T: Into<String>>(mut self, key: T) -> Self {
        self.public_key = Some(key.into());
        self
    }

    pub fn generate_keys(mut self) -> Self {
        let (private_key, public_key) = generate_rsa_keys();
        self.private_key = Some(private_key);
        self.public_key = Some(public_key);
        self
    }

    pub fn create_or_update(self, conn: &DbConn) -> Result<ExternalAccountModel> {
        conn.transaction(|| {
            let mut handle = match ExternalAccountModel::get_from_username(conn, &self.username) {
                Ok(entry) => entry,
                Err(_) => {
                    let entry = self.build();
                    ExternalAccountModel::create(conn, &entry)?
                }
            };

            handle.public_key = match self.public_key {
                Some(k) => k,
                None => handle.public_key,
            };
            handle.private_key = match self.private_key {
                Some(k) => k,
                None => handle.private_key,
            };
            handle.save(conn)?;

            Ok(handle)
        })
    }

    fn build(&self) -> ExternalAccountModel {
        ExternalAccountModel {
            username: self.username.clone(),
            public_key: self.public_key.clone().unwrap_or_else(String::new),
            private_key: self.private_key.clone().unwrap_or_else(String::new),
        }
    }
}

impl ExternalAccountModel {
    fn create(conn: &DbConn, entry: &Self) -> Result<Self> {
        diesel::insert_into(external_account::table)
            .values(entry)
            .get_result(conn)
            .map_err(Into::into)
    }

    /// Create builder.
    ///
    /// # Arguments
    ///
    /// * `username` - Username
    pub fn builder(username: &str) -> ExternalAccountModelBuilder {
        ExternalAccountModelBuilder::default(username)
    }

    /// Create builder from model.
    ///
    /// # Arguments
    ///
    /// * `username` - Username
    pub fn builder_from_model(model: &Self) -> ExternalAccountModelBuilder {
        ExternalAccountModelBuilder::from_model(model)
    }

    /// Generate access token.
    pub fn generate_access_token(&self) -> Result<String> {
        let now_ts = now();
        let claims = ExternalJwtClaims {
            // Issued at time
            iat: now_ts,
            // Username
            iss: self.username.clone(),
        };

        create_jwt(&self.private_key, &claims).map_err(Into::into)
    }

    /// Get external account from username.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `username` - Account username
    pub fn get_from_username(conn: &DbConn, username: &str) -> Result<Self> {
        external_account::table
            .filter(external_account::username.eq(username))
            .first(conn)
            .map_err(|_e| DatabaseError::UnknownExternalAccount(username.to_string()))
    }

    /// List all external accounts.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    pub fn list(conn: &DbConn) -> Result<Vec<Self>> {
        external_account::table
            .load::<Self>(conn)
            .map_err(Into::into)
    }

    /// Remove external account.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    pub fn remove(&self, conn: &DbConn) -> Result<()> {
        diesel::delete(
            external_account::table.filter(external_account::username.eq(&self.username)),
        )
        .execute(conn)
        .map_err(Into::into)
        .map(|_| ())
    }

    /// Save model instance to database.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    pub fn save(&mut self, conn: &DbConn) -> Result<()> {
        self.save_changes::<Self>(conn)
            .map_err(Into::into)
            .map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_conf::Config;
    use pretty_assertions::assert_eq;

    use crate::establish_single_test_connection;

    use super::*;

    fn test_init() -> (Config, DbConn) {
        let config = Config::from_env();
        let conn = establish_single_test_connection(&config).unwrap();

        (config, conn)
    }

    #[test]
    fn create_and_update() {
        let (_, conn) = test_init();

        let acc = ExternalAccountModel::builder("ext1")
            .create_or_update(&conn)
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
            .create_or_update(&conn)
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
            .create_or_update(&conn)
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
        assert_eq!(ExternalAccountModel::list(&conn).unwrap().len(), 1);
    }
}
