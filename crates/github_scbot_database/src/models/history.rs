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
    /// Username.
    pub username: String,
    /// Received at.
    pub received_at: chrono::NaiveDateTime,
    /// Event key.
    pub event_key: String,
    /// Payload.
    pub payload: String,
}

/// History webhook creation.
#[derive(Debug, Insertable)]
#[table_name = "history_webhook"]
pub struct HistoryWebhookCreation {
    /// Repository ID.
    pub repository_id: i32,
    /// Pull request ID.
    pub pull_request_id: i32,
    /// Username.
    pub username: String,
    /// Received at.
    pub received_at: chrono::NaiveDateTime,
    /// Event key.
    pub event_key: String,
    /// Payload.
    pub payload: String,
}

impl HistoryWebhookModel {
    /// Create history webhook entry.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `entry` - Creation entry
    pub fn create(conn: &DbConn, entry: HistoryWebhookCreation) -> Result<Self> {
        diesel::insert_into(history_webhook::table)
            .values(&entry)
            .get_result(conn)
            .map_err(Into::into)
    }

    /// Create history webhook entry from values.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `repo` - Repository
    /// * `pr` - Pull request
    /// * `username` - Username
    /// * `received_at` - Received at
    /// * `event_key` - Event key
    /// * `payload` - Payload
    pub fn create_for_time(
        conn: &DbConn,
        repo: &RepositoryModel,
        pr: &PullRequestModel,
        username: &str,
        received_at: chrono::NaiveDateTime,
        event_key: EventType,
        payload: &str,
    ) -> Result<Self> {
        Self::create(
            conn,
            HistoryWebhookCreation {
                repository_id: repo.id,
                pull_request_id: pr.id,
                username: username.into(),
                received_at,
                event_key: event_key.to_str().into(),
                payload: payload.into(),
            },
        )
    }

    /// Create history webhook entry from values, for now.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `repo` - Repository
    /// * `pr` - Pull request
    /// * `username` - Username
    /// * `event_key` - Event key
    /// * `payload` - Payload
    pub fn create_for_now<T: Serialize>(
        conn: &DbConn,
        repo: &RepositoryModel,
        pr: &PullRequestModel,
        username: &str,
        event_key: EventType,
        payload: &T,
    ) -> Result<Self> {
        Self::create(
            conn,
            HistoryWebhookCreation {
                repository_id: repo.id,
                pull_request_id: pr.id,
                username: username.into(),
                received_at: chrono::Utc::now().naive_utc(),
                event_key: event_key.to_str().into(),
                payload: serde_json::to_string_pretty(payload).unwrap(),
            },
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
