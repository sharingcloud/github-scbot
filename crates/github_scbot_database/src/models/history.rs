//! History models.

use diesel::prelude::*;
use github_scbot_types::events::EventType;
use serde::{Deserialize, Serialize};

use crate::{schema::history_webhook, DbConn, Result};

use super::{PullRequestModel, RepositoryModel};

/// History webhook model.
#[derive(Debug, Deserialize, Serialize, Queryable, Identifiable, Clone, PartialEq, Eq)]
#[table_name = "history_webhook"]
pub struct HistoryWebhookModel {
    /// Database ID.
    pub id: i32,
    /// Repository ID.
    pub repository_id: i32,
    /// Pull request ID.
    pub pull_request_id: i32,
    /// Received at.
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
struct HistoryWebhookCreation {
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

#[must_use]
pub struct HistoryWebhookModelBuilder<'a> {
    repo_model: &'a RepositoryModel,
    pr_model: &'a PullRequestModel,
    username: String,
    received_at: chrono::NaiveDateTime,
    event_key: EventType,
    payload: String,
}

impl<'a> HistoryWebhookModelBuilder<'a> {
    pub fn default(repo_model: &'a RepositoryModel, pr_model: &'a PullRequestModel) -> Self {
        Self {
            repo_model,
            pr_model,
            username: String::new(),
            received_at: chrono::Utc::now().naive_utc(),
            event_key: EventType::Ping,
            payload: String::new(),
        }
    }

    pub fn username<T: Into<String>>(mut self, username: T) -> Self {
        self.username = username.into();
        self
    }

    pub fn payload<T: Serialize>(mut self, payload: &T) -> Self {
        self.payload = serde_json::to_string_pretty(payload).unwrap();
        self
    }

    pub fn received_at<T: Into<chrono::NaiveDateTime>>(mut self, received_at: T) -> Self {
        self.received_at = received_at.into();
        self
    }

    pub fn event_key<T: Into<EventType>>(mut self, event_key: T) -> Self {
        self.event_key = event_key.into();
        self
    }

    fn build(self) -> HistoryWebhookModel {
        HistoryWebhookModel {
            id: -1,
            repository_id: self.repo_model.id,
            pull_request_id: self.pr_model.id,
            username: self.username,
            event_key: self.event_key.to_str().into(),
            received_at: self.received_at,
            payload: self.payload,
        }
    }

    pub fn create(self, conn: &DbConn) -> Result<HistoryWebhookModel> {
        HistoryWebhookModel::create(conn, self.build().into())
    }
}

impl HistoryWebhookModel {
    /// Create builder.
    ///
    /// # Arguments
    ///
    /// * `repo_model` - Repository
    /// * `pr_model` - Pull request
    pub fn builder<'a>(
        repo_model: &'a RepositoryModel,
        pr_model: &'a PullRequestModel,
    ) -> HistoryWebhookModelBuilder<'a> {
        HistoryWebhookModelBuilder::default(repo_model, pr_model)
    }

    fn create(conn: &DbConn, entry: HistoryWebhookCreation) -> Result<Self> {
        diesel::insert_into(history_webhook::table)
            .values(&entry)
            .get_result(conn)
            .map_err(Into::into)
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

    /// List all entries.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    pub fn list(conn: &DbConn) -> Result<Vec<Self>> {
        history_webhook::table
            .load::<Self>(conn)
            .map_err(Into::into)
    }

    /// List entries from repository id.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `repository_id` - Repository ID
    pub fn list_from_repository_id(conn: &DbConn, repository_id: i32) -> Result<Vec<Self>> {
        history_webhook::table
            .filter(history_webhook::repository_id.eq(repository_id))
            .load::<Self>(conn)
            .map_err(Into::into)
    }

    /// Remove all entries.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    pub fn remove_all(conn: &DbConn) -> Result<()> {
        diesel::delete(history_webhook::table).execute(conn)?;

        Ok(())
    }
}
