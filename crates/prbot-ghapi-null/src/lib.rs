//! Null driver for GH API.

#![warn(missing_docs)]
#![warn(clippy::all)]

use async_trait::async_trait;
use prbot_ghapi_interface::{
    gif::GifResponse,
    review::GhReviewApi,
    types::{
        GhCheckRun, GhCommitStatus, GhCommitStatusState, GhMergeStrategy, GhPullRequest,
        GhReactionType, GhUser, GhUserPermission,
    },
    ApiService, Result,
};

/// Null API service.
#[derive(Clone, Default)]
pub struct NullApiService {
    _private: (),
}

impl NullApiService {
    /// Build a null API service.
    pub fn new() -> Self {
        Self { _private: () }
    }
}

#[async_trait]
impl ApiService for NullApiService {
    #[tracing::instrument(skip(self), ret)]
    async fn issue_labels_list(
        &self,
        owner: &str,
        name: &str,
        _issue_number: u64,
    ) -> Result<Vec<String>> {
        Ok(vec![])
    }

    #[tracing::instrument(skip(self))]
    async fn issue_labels_replace_all(
        &self,
        owner: &str,
        name: &str,
        _issue_number: u64,
        labels: &[String],
    ) -> Result<()> {
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn issue_labels_add(
        &self,
        owner: &str,
        name: &str,
        _issue_number: u64,
        labels: &[String],
    ) -> Result<()> {
        Ok(())
    }

    #[tracing::instrument(skip(self), ret)]
    async fn user_permissions_get(
        &self,
        owner: &str,
        name: &str,
        username: &str,
    ) -> Result<GhUserPermission> {
        Ok(GhUserPermission::Write)
    }

    #[tracing::instrument(skip(self))]
    async fn check_runs_list(
        &self,
        owner: &str,
        name: &str,
        git_ref: &str,
    ) -> Result<Vec<GhCheckRun>> {
        Ok(vec![])
    }

    #[tracing::instrument(skip(self), ret)]
    async fn comments_post(
        &self,
        owner: &str,
        name: &str,
        _issue_number: u64,
        body: &str,
    ) -> Result<u64> {
        Ok(1)
    }

    #[tracing::instrument(skip(self), ret)]
    async fn comments_update(
        &self,
        owner: &str,
        name: &str,
        _comment_id: u64,
        body: &str,
    ) -> Result<u64> {
        Ok(1)
    }

    #[tracing::instrument(skip(self))]
    async fn comments_delete(&self, owner: &str, name: &str, _comment_id: u64) -> Result<()> {
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn comment_reactions_add(
        &self,
        owner: &str,
        name: &str,
        _comment_id: u64,
        _reaction_type: GhReactionType,
    ) -> Result<()> {
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn pulls_get(&self, owner: &str, name: &str, number: u64) -> Result<GhPullRequest> {
        Ok(GhPullRequest {
            number,
            user: GhUser {
                login: owner.into(),
            },
            ..Default::default()
        })
    }

    #[tracing::instrument(skip(self))]
    async fn pulls_merge(
        &self,
        owner: &str,
        name: &str,
        _number: u64,
        commit_title: &str,
        commit_message: &str,
        _merge_strategy: GhMergeStrategy,
    ) -> Result<()> {
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn pull_reviewer_requests_add(
        &self,
        owner: &str,
        name: &str,
        _number: u64,
        reviewers: &[String],
    ) -> Result<()> {
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn pull_reviewer_requests_remove(
        &self,
        owner: &str,
        name: &str,
        _number: u64,
        reviewers: &[String],
    ) -> Result<()> {
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn pull_reviews_list(
        &self,
        owner: &str,
        name: &str,
        _number: u64,
    ) -> Result<Vec<GhReviewApi>> {
        Ok(vec![])
    }

    #[tracing::instrument(skip(self))]
    async fn commit_statuses_combined(
        &self,
        owner: &str,
        name: &str,
        git_ref: &str,
    ) -> Result<GhCommitStatus> {
        Ok(GhCommitStatus {
            state: GhCommitStatusState::Success,
            items: vec![],
        })
    }

    #[tracing::instrument(skip(self))]
    async fn commit_statuses_update(
        &self,
        owner: &str,
        name: &str,
        git_ref: &str,
        _status: GhCommitStatusState,
        title: &str,
        body: &str,
    ) -> Result<()> {
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn gif_search(&self, api_key: &str, search: &str) -> Result<GifResponse> {
        Ok(GifResponse { results: vec![] })
    }

    async fn installations_create_token(
        &self,
        _auth_token: &str,
        _installation_id: u64,
    ) -> Result<String> {
        Ok("token".into())
    }
}
