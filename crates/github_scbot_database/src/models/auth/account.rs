use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{schema::account, DatabaseError, DbConn, Result};

/// Account model.
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
#[table_name = "account"]
pub struct AccountModel {
    /// Username.
    pub username: String,
    /// Is admin?
    pub is_admin: bool,
}

#[must_use]
pub struct AccountModelBuilder {
    username: String,
    is_admin: Option<bool>,
}

impl AccountModelBuilder {
    pub fn default<T: Into<String>>(username: T) -> Self {
        Self {
            username: username.into(),
            is_admin: None,
        }
    }

    pub fn from_model(model: &AccountModel) -> Self {
        Self {
            username: model.username.clone(),
            is_admin: Some(model.is_admin),
        }
    }

    pub fn admin(mut self, value: bool) -> Self {
        self.is_admin = Some(value);
        self
    }

    pub fn create_or_update(self, conn: &DbConn) -> Result<AccountModel> {
        conn.transaction(|| {
            let mut handle = match AccountModel::get_from_username(conn, &self.username) {
                Ok(entry) => entry,
                Err(_) => {
                    let entry = self.build();
                    AccountModel::create(conn, &entry)?
                }
            };

            handle.is_admin = match self.is_admin {
                Some(a) => a,
                None => handle.is_admin,
            };
            handle.save(conn)?;

            Ok(handle)
        })
    }

    fn build(&self) -> AccountModel {
        AccountModel {
            username: self.username.clone(),
            is_admin: self.is_admin.unwrap_or(false),
        }
    }
}

impl AccountModel {
    /// Create builder.
    ///
    /// # Arguments
    ///
    /// * `username` - Username
    pub fn builder<T: Into<String>>(username: T) -> AccountModelBuilder {
        AccountModelBuilder::default(username)
    }

    /// Create builder from model.
    ///
    /// # Arguments
    ///
    /// * `model` - Model
    pub fn builder_from_model(model: &Self) -> AccountModelBuilder {
        AccountModelBuilder::from_model(model)
    }

    fn create(conn: &DbConn, entry: &Self) -> Result<Self> {
        diesel::insert_into(account::table)
            .values(entry)
            .get_result(conn)
            .map_err(Into::into)
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

        let acc = AccountModel::builder("acc")
            .create_or_update(&conn)
            .unwrap();

        assert_eq!(
            acc,
            AccountModel {
                username: "acc".into(),
                is_admin: false
            }
        );

        let acc = AccountModel::builder("acc")
            .admin(true)
            .create_or_update(&conn)
            .unwrap();

        assert_eq!(
            acc,
            AccountModel {
                username: "acc".into(),
                is_admin: true
            }
        );

        // Only one account after 2 create_or_update.
        assert_eq!(AccountModel::list(&conn).unwrap().len(), 1);
    }
}
