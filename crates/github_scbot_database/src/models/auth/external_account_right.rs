use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::models::RepositoryModel;
use crate::{schema::external_account_right, DatabaseError, DbConn, Result};

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
