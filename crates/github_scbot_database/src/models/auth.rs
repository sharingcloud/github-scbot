//! Authentication models.

use diesel::prelude::*;
use github_scbot_crypto::{create_jwt, generate_rsa_keys, now};
use serde::{Deserialize, Serialize};

use super::RepositoryModel;
use crate::{
    schema::{external_account, external_account_right},
    DatabaseError, DbConn, Result,
};

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
    /// Create external account with keys.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `username` - Username
    /// * `public_key` - Public key
    /// * `private_key` - Private key
    pub fn create_with_keys(
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
    pub fn create(conn: &DbConn, username: &str) -> Result<Self> {
        match Self::get_from_username(conn, username) {
            Ok(account) => Ok(account),
            Err(_) => {
                let (private_key, public_key) = generate_rsa_keys();
                Self::create_with_keys(conn, username, &public_key, &private_key)
            }
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

/// External account right.
#[derive(Debug, Deserialize, Serialize, Queryable, Identifiable, Clone, Insertable)]
#[primary_key(username, repository_id)]
#[table_name = "external_account_right"]
pub struct ExternalAccountRightModel {
    /// Username.
    pub username: String,
    /// Repository ID.
    pub repository_id: i32,
}

impl ExternalAccountRightModel {
    /// Add right to external account.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `username` - Username
    /// * `repository_id` - Repository ID
    pub fn add_right(conn: &DbConn, username: &str, repository_id: i32) -> Result<Self> {
        match Self::get_right(conn, username, repository_id) {
            Ok(existing) => Ok(existing),
            Err(_) => {
                let entry = Self {
                    username: username.into(),
                    repository_id,
                };

                let data = diesel::insert_into(external_account_right::table)
                    .values(&entry)
                    .get_result(conn)?;

                Ok(data)
            }
        }
    }

    /// Remove right from external account.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `username` - Username
    /// * `repository_id` - Repository ID
    pub fn remove_right(conn: &DbConn, username: &str, repository_id: i32) -> Result<()> {
        diesel::delete(
            external_account_right::table
                .filter(external_account_right::username.eq(username))
                .filter(external_account_right::repository_id.eq(repository_id)),
        )
        .execute(conn)?;

        Ok(())
    }

    /// Remove all rights from external account.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `username` - Username
    /// * `repository_id` - Repository ID
    pub fn remove_rights(conn: &DbConn, username: &str) -> Result<()> {
        diesel::delete(
            external_account_right::table.filter(external_account_right::username.eq(username)),
        )
        .execute(conn)?;

        Ok(())
    }

    /// Get right from external account.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `username` - Username
    /// * `repository_id` - Repository ID
    pub fn get_right(conn: &DbConn, username: &str, repository_id: i32) -> Result<Self> {
        external_account_right::table
            .filter(external_account_right::username.eq(username))
            .filter(external_account_right::repository_id.eq(repository_id))
            .first(conn)
            .map_err(|_e| {
                DatabaseError::UnknownExternalAccountRight(
                    username.to_string(),
                    format!("<ID {}>", repository_id),
                )
            })
    }

    /// List rights from external account.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `username` - Username
    pub fn list_rights(conn: &DbConn, username: &str) -> Result<Vec<Self>> {
        external_account_right::table
            .filter(external_account_right::username.eq(username))
            .get_results(conn)
            .map_err(Into::into)
    }

    /// Get repository from right.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    pub fn get_repository(&self, conn: &DbConn) -> Result<RepositoryModel> {
        RepositoryModel::get_from_id(conn, self.repository_id)
    }

    /// List all external account rights.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    pub fn list(conn: &DbConn) -> Result<Vec<Self>> {
        external_account_right::table
            .load::<Self>(conn)
            .map_err(Into::into)
    }
}
