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

impl ExternalAccountModel {
    /// Create external account.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `username` - Username
    /// * `public_key` - Public key
    /// * `private_key` - Private key
    pub fn create(
        conn: &DbConn,
        username: &str,
        public_key: &str,
        private_key: &str,
    ) -> Result<Self> {
        let entry = Self {
            username: username.into(),
            public_key: public_key.into(),
            private_key: private_key.into(),
        };

        let data = diesel::insert_into(external_account::table)
            .values(&entry)
            .get_result(conn)?;
        Ok(data)
    }

    /// Create external account, generating keys.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `username` - Account username
    pub fn create_with_keys(conn: &DbConn, username: &str) -> Result<Self> {
        let (private_key, public_key) = generate_rsa_keys();
        Self::create(conn, username, &public_key, &private_key)
    }

    /// Get or create external account.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `username` - Account username
    /// * `public_key` - Public key
    /// * `private_key` - Private key
    pub fn get_or_create(
        conn: &DbConn,
        username: &str,
        public_key: &str,
        private_key: &str,
    ) -> Result<Self> {
        match Self::get_from_username(conn, username) {
            Ok(v) => Ok(v),
            Err(_) => Self::create(conn, username, public_key, private_key),
        }
    }

    /// Get or create external account with autogenerated keys.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `username` - Account username
    pub fn get_or_create_with_keys(conn: &DbConn, username: &str) -> Result<Self> {
        match Self::get_from_username(conn, &username) {
            Ok(v) => Ok(v),
            Err(_) => Self::create_with_keys(conn, &username),
        }
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
        .execute(conn)?;

        Ok(())
    }

    /// Save model instance to database.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    pub fn save(&mut self, conn: &DbConn) -> Result<()> {
        self.save_changes::<Self>(conn)?;

        Ok(())
    }
}