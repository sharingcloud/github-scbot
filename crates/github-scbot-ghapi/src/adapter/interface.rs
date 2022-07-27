use std::collections::HashMap;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use github_scbot_core::types::{
    checks::GhCheckSuite,
    common::{GhUser, GhUserPermission},
    issues::GhReactionType,
    pulls::{GhMergeStrategy, GhPullRequest},
    reviews::GhReviewState,
    status::StatusState,
};
use serde::{Deserialize, Serialize};

use crate::Result;

/// Review state (API version)
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GhReviewStateApi {
    /// Approved.
    Approved,
    /// Changes requested.
    ChangesRequested,
    /// Commented.
    Commented,
    /// Dismissed.
    Dismissed,
    /// Pending.
    Pending,
}

impl From<GhReviewStateApi> for GhReviewState {
    fn from(state_api: GhReviewStateApi) -> Self {
        use heck::SnakeCase;

        let str_value = serde_plain::to_string(&state_api).unwrap();
        let snake_case_value = str_value.to_snake_case();
        serde_plain::from_str(&snake_case_value).unwrap()
    }
}

/// Review (API version)
#[derive(Deserialize, Clone, Debug)]
pub struct GhReviewApi {
    /// User.
    pub user: GhUser,
    /// Submitted at.
    pub submitted_at: DateTime<Utc>,
    /// State.
    pub state: GhReviewStateApi,
}

impl Default for GhReviewApi {
    fn default() -> Self {
        Self {
            user: GhUser::default(),
            submitted_at: Utc::now(),
            state: GhReviewStateApi::Pending,
        }
    }
}

/// Gif format.
#[allow(non_camel_case_types)]
#[derive(Deserialize, PartialEq, Eq, Hash, Clone, Copy, Debug)]
#[serde(rename_all = "lowercase")]
pub enum GifFormat {
    /// Standard GIF.
    Gif,
    /// Medium GIF.
    MediumGif,
    /// Tiny GIF.
    TinyGif,
    /// Nano GIF.
    NanoGif,
    /// MP4.
    Mp4,
    /// Looped MP4.
    LoopedMp4,
    /// Tiny MP4.
    TinyMp4,
    /// Nano MP4.
    NanoMp4,
    /// WebM.
    WebM,
    /// Tiny WebM.
    TinyWebM,
    /// Nano WebM.
    NanoWebM,
    /// Transparent WebP.
    WebP_Transparent,
}

/// Media object.
#[derive(Deserialize, Clone, Debug)]
pub struct MediaObject {
    /// Media URL.
    pub url: String,
    /// Media size.
    pub size: Option<usize>,
}

/// Gif object.
#[derive(Deserialize, Clone, Default, Debug)]
pub struct GifObject {
    /// Media dict.
    pub media: Vec<HashMap<GifFormat, MediaObject>>,
}

/// Gif response.
#[derive(Deserialize, Clone, Default, Debug)]
pub struct GifResponse {
    /// Results.
    pub results: Vec<GifObject>,
}

/// GitHub API Adapter interface
#[mockall::automock]
#[async_trait(?Send)]
pub trait ApiService: Send + Sync {
    /// List labels from a target issue.
    async fn issue_labels_list(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
    ) -> Result<Vec<String>>;
    /// Replace all labels for a target issue.
    async fn issue_labels_replace_all(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
        labels: &[String],
    ) -> Result<()>;
    /// Add labels for a target issue.
    async fn issue_labels_add(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
        labels: &[String],
    ) -> Result<()>;
    /// Remove labels for a target issue.
    async fn issue_labels_remove(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
        labels: &[String],
    ) -> Result<()> {
        let known_labels = self.issue_labels_list(owner, name, issue_number).await?;
        let all_labels = known_labels
            .into_iter()
            .filter(|x| !labels.contains(x))
            .collect::<Vec<_>>();
        self.issue_labels_replace_all(owner, name, issue_number, &all_labels)
            .await
    }
    /// Get user permissions from a repository.
    async fn user_permissions_get(
        &self,
        owner: &str,
        name: &str,
        username: &str,
    ) -> Result<GhUserPermission>;
    /// List check suites from a repository.
    async fn check_suites_list(
        &self,
        owner: &str,
        name: &str,
        git_ref: &str,
    ) -> Result<Vec<GhCheckSuite>>;
    /// Post a comment on a pull request.
    async fn comments_post(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
        body: &str,
    ) -> Result<u64>;
    /// Update a comment on a pull request.
    async fn comments_update(
        &self,
        owner: &str,
        name: &str,
        comment_id: u64,
        body: &str,
    ) -> Result<u64>;
    /// Delete a comment on a pull request.
    async fn comments_delete(&self, owner: &str, name: &str, comment_id: u64) -> Result<()>;
    /// Add a reaction to a pull request comment.
    async fn comment_reactions_add(
        &self,
        owner: &str,
        name: &str,
        comment_id: u64,
        reaction_type: GhReactionType,
    ) -> Result<()>;
    /// Get a pull request from its number.
    async fn pulls_get(&self, owner: &str, name: &str, issue_number: u64) -> Result<GhPullRequest>;
    /// Merge a pull request.
    async fn pulls_merge(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
        commit_title: &str,
        commit_message: &str,
        merge_strategy: GhMergeStrategy,
    ) -> Result<()>;
    /// Add reviewers to a pull request.
    async fn pull_reviewer_requests_add(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
        reviewers: &[String],
    ) -> Result<()>;
    /// Remove reviewers from a pull request.
    async fn pull_reviewer_requests_remove(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
        reviewers: &[String],
    ) -> Result<()>;
    /// List reviews from a pull request.
    async fn pull_reviews_list(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
    ) -> Result<Vec<GhReviewApi>>;
    /// Update commit status.
    async fn commit_statuses_update(
        &self,
        owner: &str,
        name: &str,
        git_ref: &str,
        status: StatusState,
        title: &str,
        body: &str,
    ) -> Result<()>;
    /// Search a GIF.
    async fn gif_search(&self, api_key: &str, search: &str) -> Result<GifResponse>;
    /// Create installation token.
    async fn installations_create_token(
        &self,
        auth_token: &str,
        installation_id: u64,
    ) -> Result<String>;
}
