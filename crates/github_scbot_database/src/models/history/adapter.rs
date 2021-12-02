use async_trait::async_trait;
use diesel::prelude::*;
use github_scbot_utils::Mock;
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
pub struct HistoryWebhookDbAdapter {
    pool: DbPool,
}

impl HistoryWebhookDbAdapter {
    /// Creates a new history webhook DB adapter.
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl IHistoryWebhookDbAdapter for HistoryWebhookDbAdapter {
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
pub struct DummyHistoryWebhookDbAdapter {
    /// Create response.
    pub create_response: Mock<HistoryWebhookCreation, Result<HistoryWebhookModel>>,
    /// List response.
    pub list_response: Mock<(), Result<Vec<HistoryWebhookModel>>>,
    /// List from repository ID response.
    pub list_from_repository_id_response: Mock<i32, Result<Vec<HistoryWebhookModel>>>,
    /// Remove all response.
    pub remove_all_response: Mock<(), Result<()>>,
}

impl Default for DummyHistoryWebhookDbAdapter {
    fn default() -> Self {
        Self {
            create_response: Mock::new(Box::new(|e| Ok(e.into()))),
            list_response: Mock::new(Box::new(|_| Ok(Vec::new()))),
            list_from_repository_id_response: Mock::new(Box::new(|_| Ok(Vec::new()))),
            remove_all_response: Mock::new(Box::new(|_| Ok(()))),
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
        self.create_response.call(entry)
    }

    async fn list(&self) -> Result<Vec<HistoryWebhookModel>> {
        self.list_response.call(())
    }

    async fn list_from_repository_id(
        &self,
        repository_id: i32,
    ) -> Result<Vec<HistoryWebhookModel>> {
        self.list_from_repository_id_response.call(repository_id)
    }

    async fn remove_all(&self) -> Result<()> {
        self.remove_all_response.call(())
    }
}
