//! History models.

use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use super::{PullRequestModel, RepositoryModel};
use crate::schema::history_webhook;

mod adapter;
mod builder;
pub use adapter::{
    DummyHistoryWebhookDbAdapter, HistoryWebhookDbAdapter, IHistoryWebhookDbAdapter,
};
use builder::HistoryWebhookModelBuilder;

/// History webhook model.
#[derive(
    Debug, Deserialize, Serialize, Queryable, Identifiable, Clone, PartialEq, Eq, SmartDefault,
)]
#[table_name = "history_webhook"]
pub struct HistoryWebhookModel {
    /// Database ID.
    pub id: i32,
    /// Repository ID.
    pub repository_id: i32,
    /// Pull request ID.
    pub pull_request_id: i32,
    /// Received at.
    #[default(chrono::Utc::now().naive_utc())]
    pub received_at: chrono::NaiveDateTime,
    /// Username.
    pub username: String,
    /// Event key.
    pub event_key: String,
    /// Payload.
    pub payload: String,
}

#[derive(Debug, Insertable)]
#[table_name = "history_webhook"]
pub struct HistoryWebhookCreation {
    pub repository_id: i32,
    pub pull_request_id: i32,
    pub received_at: chrono::NaiveDateTime,
    pub username: String,
    pub event_key: String,
    pub payload: String,
}

impl From<HistoryWebhookModel> for HistoryWebhookCreation {
    fn from(model: HistoryWebhookModel) -> Self {
        Self {
            repository_id: model.repository_id,
            pull_request_id: model.pull_request_id,
            username: model.username,
            received_at: model.received_at,
            event_key: model.event_key,
            payload: model.payload,
        }
    }
}

impl HistoryWebhookModel {
    /// Create builder.
    pub fn builder<'a>(
        repo_model: &'a RepositoryModel,
        pr_model: &'a PullRequestModel,
    ) -> HistoryWebhookModelBuilder<'a> {
        HistoryWebhookModelBuilder::default(repo_model, pr_model)
    }

    /// Show entry.
    pub fn show(&self) {
        println!(
            "[Repo <ID {repo_id}> | PR <ID {pr_id}>]: {event}\n{value}",
            repo_id = self.repository_id,
            pr_id = self.pull_request_id,
            event = self.event_key,
            value = self.payload
        )
    }
}
