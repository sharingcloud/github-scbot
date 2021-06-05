use async_trait::async_trait;
use diesel::prelude::*;
use tokio_diesel::AsyncRunQueryDsl;

use super::ExternalAccountModel;
use crate::{schema::external_account, DatabaseError, DbPool, Result};

/// External account DB adapter.
#[async_trait]
pub trait IExternalAccountDbAdapter {
    /// Creates a new external account.
    async fn create(&self, entry: ExternalAccountModel) -> Result<ExternalAccountModel>;
    /// Gets an external account from username.
    async fn get_from_username(&self, username: &str) -> Result<ExternalAccountModel>;
    /// Lists available external accounts.
    async fn list(&self) -> Result<Vec<ExternalAccountModel>>;
    /// Removes a specific external account.
    async fn remove(&self, entry: ExternalAccountModel) -> Result<()>;
    /// Saves and updates a specific external account.
    async fn save(&self, entry: &mut ExternalAccountModel) -> Result<()>;
}

/// Concrete external account DB adapter.
pub struct ExternalAccountDbAdapter<'a> {
    pool: &'a DbPool,
}

impl<'a> ExternalAccountDbAdapter<'a> {
    /// Creates a new external account DB adapter.
    pub fn new(pool: &'a DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<'a> IExternalAccountDbAdapter for ExternalAccountDbAdapter<'a> {
    async fn create(&self, entry: ExternalAccountModel) -> Result<ExternalAccountModel> {
        diesel::insert_into(external_account::table)
            .values(entry)
            .get_result_async(&self.pool)
            .await
            .map_err(DatabaseError::from)
    }

    async fn get_from_username(&self, username: &str) -> Result<ExternalAccountModel> {
        let username = username.to_owned();

        external_account::table
            .filter(external_account::username.eq(username.clone()))
            .first_async(&self.pool)
            .await
            .map_err(|_e| DatabaseError::UnknownExternalAccount(username))
    }

    async fn list(&self) -> Result<Vec<ExternalAccountModel>> {
        external_account::table
            .load_async::<ExternalAccountModel>(&self.pool)
            .await
            .map_err(DatabaseError::from)
    }

    async fn remove(&self, entry: ExternalAccountModel) -> Result<()> {
        diesel::delete(
            external_account::table.filter(external_account::username.eq(entry.username)),
        )
        .execute_async(&self.pool)
        .await
        .map_err(DatabaseError::from)
        .map(|_| ())
    }

    async fn save(&self, entry: &mut ExternalAccountModel) -> Result<()> {
        let copy = entry.clone();

        *entry = diesel::update(
            external_account::table.filter(external_account::username.eq(copy.username.clone())),
        )
        .set(copy)
        .get_result_async(&self.pool)
        .await
        .map_err(DatabaseError::from)?;

        Ok(())
    }
}

/// Dummy external account DB adapter.
#[derive(Clone)]
pub struct DummyExternalAccountDbAdapter {
    /// Create response.
    pub create_response: Result<ExternalAccountModel>,
    /// Get from username response.
    pub get_from_username_response: Result<ExternalAccountModel>,
    /// List response.
    pub list_response: Result<Vec<ExternalAccountModel>>,
    /// Remove response.
    pub remove_response: Result<()>,
    /// Save response.
    pub save_response: Result<()>,
}

impl Default for DummyExternalAccountDbAdapter {
    fn default() -> Self {
        Self {
            create_response: Ok(ExternalAccountModel::default()),
            get_from_username_response: Ok(ExternalAccountModel::default()),
            list_response: Ok(Vec::new()),
            remove_response: Ok(()),
            save_response: Ok(()),
        }
    }
}

impl DummyExternalAccountDbAdapter {
    /// Creates a new dummy external account DB adapter.
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
#[allow(unused_variables)]
impl IExternalAccountDbAdapter for DummyExternalAccountDbAdapter {
    async fn create(&self, entry: ExternalAccountModel) -> Result<ExternalAccountModel> {
        self.create_response.clone()
    }

    async fn get_from_username(&self, username: &str) -> Result<ExternalAccountModel> {
        self.get_from_username_response.clone()
    }

    async fn list(&self) -> Result<Vec<ExternalAccountModel>> {
        self.list_response.clone()
    }

    async fn remove(&self, entry: ExternalAccountModel) -> Result<()> {
        self.remove_response.clone()
    }

    async fn save(&self, entry: &mut ExternalAccountModel) -> Result<()> {
        self.save_response.clone()
    }
}
