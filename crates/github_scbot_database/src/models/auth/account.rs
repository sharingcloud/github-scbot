use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{schema::account, DatabaseError, DbConn, Result};

/// Account model.
#[derive(
    Debug, Deserialize, Insertable, Identifiable, Serialize, Queryable, Clone, AsChangeset,
)]
#[primary_key(username)]
#[table_name = "account"]
pub struct AccountModel {
    /// Username.
    pub username: String,
    /// Is admin?
    pub is_admin: bool,
}

impl AccountModel {
    /// Create account.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `username` - Account username
    /// * `is_admin` - Is administrator?
    pub fn create(conn: &DbConn, username: &str, is_admin: bool) -> Result<Self> {
        let entry = Self {
            username: username.into(),
            is_admin,
        };

        let data = diesel::insert_into(account::table)
            .values(&entry)
            .get_result(conn)?;
        Ok(data)
    }

    /// Get or create account.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `username` - Account username
    /// * `is_admin` - Is administrator?
    pub fn get_or_create(conn: &DbConn, username: &str, is_admin: bool) -> Result<Self> {
        match Self::get_from_username(conn, username) {
            Err(_) => Self::create(conn, username, is_admin),
            Ok(v) => Ok(v),
        }
    }

    /// Get account from username.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `username` - Account username
    pub fn get_from_username(conn: &DbConn, username: &str) -> Result<Self> {
        account::table
            .filter(account::username.eq(username))
            .first(conn)
            .map_err(|_e| DatabaseError::UnknownAccount(username.to_string()))
    }

    /// List all accounts.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    pub fn list(conn: &DbConn) -> Result<Vec<Self>> {
        account::table.load::<Self>(conn).map_err(Into::into)
    }

    /// List admin accounts.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    pub fn list_admin_accounts(conn: &DbConn) -> Result<Vec<Self>> {
        account::table
            .filter(account::is_admin.eq(true))
            .load::<Self>(conn)
            .map_err(Into::into)
    }

    /// Remove account.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    pub fn remove(&self, conn: &DbConn) -> Result<()> {
        diesel::delete(account::table.filter(account::username.eq(&self.username)))
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
