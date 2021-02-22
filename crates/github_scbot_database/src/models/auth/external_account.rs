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
    Debug, Deserialize, Insertable, Identifiable, Serialize, Queryable, Clone, AsChangeset,
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

pub struct ExternalAccountModelBuilder {
    username: String,
    public_key: String,
    private_key: String,
}

impl ExternalAccountModelBuilder {
    pub fn default<T: Into<String>>(username: T) -> Self {
        Self {
            username: username.into(),
            public_key: String::new(),
            private_key: String::new(),
        }
    }

    pub fn username<T: Into<String>>(mut self, username: T) -> Self {
        self.username = username.into();
        self
    }

    pub fn private_key<T: Into<String>>(mut self, key: T) -> Self {
        self.private_key = key.into();
        self
    }

    pub fn public_key<T: Into<String>>(mut self, key: T) -> Self {
        self.public_key = key.into();
        self
    }

    pub fn generate_keys(mut self) -> Self {
        let (private_key, public_key) = generate_rsa_keys();
        self.private_key = private_key;
        self.public_key = public_key;
        self
    }

    pub fn build(self) -> ExternalAccountModel {
        ExternalAccountModel {
            username: self.username,
            public_key: self.public_key,
            private_key: self.private_key,
        }
    }

    pub fn create(self, conn: &DbConn) -> Result<ExternalAccountModel> {
        ExternalAccountModel::create(conn, &self.build())
    }

    pub fn create_or_update(self, conn: &DbConn) -> Result<ExternalAccountModel> {
        ExternalAccountModel::create_or_update(conn, &self.build())
    }
}

impl ExternalAccountModel {
    /// Create builder.
    ///
    /// # Arguments
    ///
    /// * `username` - Username
    pub fn builder(username: &str) -> ExternalAccountModelBuilder {
        ExternalAccountModelBuilder::default(username)
    }

    /// Create model in database.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `entry` - Entry
    pub fn create(conn: &DbConn, entry: &Self) -> Result<Self> {
        diesel::insert_into(external_account::table)
            .values(entry)
            .get_result(conn)
            .map_err(Into::into)
    }

    /// Create or update model.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `entry` - Entry
    pub fn create_or_update(conn: &DbConn, entry: &Self) -> Result<Self> {
        let mut handle = match Self::get_from_username(conn, &entry.username) {
            Ok(entry) => entry,
            Err(_) => Self::create(conn, entry)?,
        };

        handle.public_key = entry.public_key.clone();
        handle.private_key = entry.private_key.clone();
        handle.save(conn)?;

        Ok(handle)
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
