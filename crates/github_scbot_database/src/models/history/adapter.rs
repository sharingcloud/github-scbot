use async_trait::async_trait;
use diesel::prelude::*;
use tokio_diesel::AsyncRunQueryDsl;

use super::{HistoryWebhookCreation, HistoryWebhookModel};
use crate::{schema::history_webhook, DatabaseError, DbPool, Result};

/// History webhook DB adapter.
#[async_trait]
pub trait IHistoryWebhookDbAdapter {
    /// Creates a new history webhook entry.
    async fn create(&self, entry: HistoryWebhookCreation) -> Result<HistoryWebhookModel>;
    /// Lists existing history webhook entries.
    async fn list(&self) -> Result<Vec<HistoryWebhookModel>>;
    /// Lists existing history webhook entries for repository.
    async fn list_from_repository_id(&self, repository_id: i32)
        -> Result<Vec<HistoryWebhookModel>>;
    /// Removes all history webhook entries.
    async fn remove_all(&self) -> Result<()>;
}

/// Concrete history webhook DB adapter.
pub struct HistoryWebhookDbAdapter<'a> {
    pool: &'a DbPool,
}

impl<'a> HistoryWebhookDbAdapter<'a> {
    /// Creates a new history webhook DB adapter.
    pub fn new(pool: &'a DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<'a> IHistoryWebhookDbAdapter for HistoryWebhookDbAdapter<'a> {
    async fn create(&self, entry: HistoryWebhookCreation) -> Result<HistoryWebhookModel> {
        diesel::insert_into(history_webhook::table)
            .values(entry)
            .get_result_async(&self.pool)
            .await
            .map_err(DatabaseError::from)
    }

    async fn list(&self) -> Result<Vec<HistoryWebhookModel>> {
        history_webhook::table
            .load_async::<HistoryWebhookModel>(&self.pool)
            .await
            .map_err(DatabaseError::from)
    }

    async fn list_from_repository_id(
        &self,
        repository_id: i32,
    ) -> Result<Vec<HistoryWebhookModel>> {
        history_webhook::table
            .filter(history_webhook::repository_id.eq(repository_id))
            .load_async::<HistoryWebhookModel>(&self.pool)
            .await
            .map_err(Into::into)
    }

    async fn remove_all(&self) -> Result<()> {
        diesel::delete(history_webhook::table)
            .execute_async(&self.pool)
            .await?;

        Ok(())
    }
}

/// Dummy history webhook DB adapter.
#[derive(Clone)]
pub struct DummyHistoryWebhookDbAdapter {
    /// Create response.
    pub create_response: Result<HistoryWebhookModel>,
    /// List response.
    pub list_response: Result<Vec<HistoryWebhookModel>>,
    /// List from repository ID response.
    pub list_from_repository_id_response: Result<Vec<HistoryWebhookModel>>,
    /// Remove all response.
    pub remove_all_response: Result<()>,
}

impl Default for DummyHistoryWebhookDbAdapter {
    fn default() -> Self {
        Self {
            create_response: Ok(HistoryWebhookModel::default()),
            list_response: Ok(Vec::new()),
            list_from_repository_id_response: Ok(Vec::new()),
            remove_all_response: Ok(()),
        }
    }
}

impl DummyHistoryWebhookDbAdapter {
    /// Creates a new dummy history webhook DB adapter.
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
#[allow(unused_variables)]
impl IHistoryWebhookDbAdapter for DummyHistoryWebhookDbAdapter {
    async fn create(&self, entry: HistoryWebhookCreation) -> Result<HistoryWebhookModel> {
        self.create_response.clone()
    }

    async fn list(&self) -> Result<Vec<HistoryWebhookModel>> {
        self.list_response.clone()
    }

    async fn list_from_repository_id(
        &self,
        repository_id: i32,
    ) -> Result<Vec<HistoryWebhookModel>> {
        self.list_from_repository_id_response.clone()
    }

    async fn remove_all(&self) -> Result<()> {
        self.remove_all_response.clone()
    }
}