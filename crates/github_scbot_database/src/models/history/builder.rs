use github_scbot_types::events::EventType;
use serde::Serialize;

use super::{HistoryWebhookModel, IHistoryWebhookDbAdapter};
use crate::{
    models::{PullRequestModel, RepositoryModel},
    Result,
};

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

    pub fn build(self) -> HistoryWebhookModel {
        HistoryWebhookModel {
            id: -1,
            repository_id: self.repo_model.id(),
            pull_request_id: self.pr_model.id(),
            username: self.username,
            event_key: self.event_key.to_str().into(),
            received_at: self.received_at,
            payload: self.payload,
        }
    }

    pub async fn create(
        self,
        db_adapter: &dyn IHistoryWebhookDbAdapter,
    ) -> Result<HistoryWebhookModel> {
        db_adapter.create(self.build().into()).await
    }
}
