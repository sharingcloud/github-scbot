//! Dummy adapter

use async_trait::async_trait;
use github_scbot_types::{
    checks::GhCheckSuite,
    common::GhUserPermission,
    issues::GhReactionType,
    pulls::{GhMergeStrategy, GhPullRequest},
    status::StatusState,
};

use super::{GhReviewApi, GifResponse, IAPIAdapter};
use crate::Result;

/// Dummy API adapter.
#[derive(Clone)]
#[allow(missing_docs)]
pub struct DummyAPIAdapter {
    pub issue_labels_list_response: Result<Vec<String>>,
    pub issue_labels_replace_all_response: Result<()>,
    pub user_permissions_get_response: Result<GhUserPermission>,
    pub check_suites_list_response: Result<Vec<GhCheckSuite>>,
    pub comments_post_response: Result<u64>,
    pub comments_update_response: Result<u64>,
    pub comments_delete_response: Result<()>,
    pub comment_reactions_add_response: Result<()>,
    pub pulls_get_response: Result<GhPullRequest>,
    pub pulls_merge_response: Result<()>,
    pub pull_reviewer_requests_add_response: Result<()>,
    pub pull_reviewer_requests_remove_response: Result<()>,
    pub pull_reviews_list_response: Result<Vec<GhReviewApi>>,
    pub commit_status_update_response: Result<()>,
    pub gif_search_response: Result<GifResponse>,
}

impl Default for DummyAPIAdapter {
    fn default() -> Self {
        Self {
            issue_labels_list_response: Ok(Vec::new()),
            issue_labels_replace_all_response: Ok(()),
            user_permissions_get_response: Ok(GhUserPermission::None),
            check_suites_list_response: Ok(Vec::new()),
            comments_post_response: Ok(0),
            comments_update_response: Ok(0),
            comments_delete_response: Ok(()),
            comment_reactions_add_response: Ok(()),
            pulls_get_response: Ok(GhPullRequest::default()),
            pulls_merge_response: Ok(()),
            pull_reviewer_requests_add_response: Ok(()),
            pull_reviewer_requests_remove_response: Ok(()),
            pull_reviews_list_response: Ok(Vec::new()),
            commit_status_update_response: Ok(()),
            gif_search_response: Ok(GifResponse::default()),
        }
    }
}

impl DummyAPIAdapter {
    /// Creates new dummy API adapter.
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
#[allow(unused_variables)]
impl IAPIAdapter for DummyAPIAdapter {
    async fn issue_labels_list(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
    ) -> Result<Vec<String>> {
        self.issue_labels_list_response.clone()
    }

    async fn issue_labels_replace_all(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
        labels: &[String],
    ) -> Result<()> {
        self.issue_labels_replace_all_response.clone()
    }

    async fn user_permissions_get(
        &self,
        owner: &str,
        name: &str,
        username: &str,
    ) -> Result<GhUserPermission> {
        self.user_permissions_get_response.clone()
    }

    async fn check_suites_list(
        &self,
        owner: &str,
        name: &str,
        git_ref: &str,
    ) -> Result<Vec<GhCheckSuite>> {
        self.check_suites_list_response.clone()
    }

    async fn comments_post(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
        body: &str,
    ) -> Result<u64> {
        self.comments_post_response.clone()
    }

    async fn comments_update(
        &self,
        owner: &str,
        name: &str,
        comment_id: u64,
        body: &str,
    ) -> Result<u64> {
        self.comments_update_response.clone()
    }

    async fn comments_delete(&self, owner: &str, name: &str, comment_id: u64) -> Result<()> {
        self.comments_delete_response.clone()
    }

    async fn comment_reactions_add(
        &self,
        owner: &str,
        name: &str,
        comment_id: u64,
        reaction_type: GhReactionType,
    ) -> Result<()> {
        self.comment_reactions_add_response.clone()
    }

    async fn pulls_get(&self, owner: &str, name: &str, issue_number: u64) -> Result<GhPullRequest> {
        self.pulls_get_response.clone()
    }

    async fn pulls_merge(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
        commit_title: &str,
        commit_message: &str,
        merge_strategy: GhMergeStrategy,
    ) -> Result<()> {
        self.pulls_merge_response.clone()
    }

    async fn pull_reviewer_requests_add(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
        reviewers: &[String],
    ) -> Result<()> {
        self.pull_reviewer_requests_add_response.clone()
    }

    async fn pull_reviewer_requests_remove(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
        reviewers: &[String],
    ) -> Result<()> {
        self.pull_reviewer_requests_remove_response.clone()
    }

    async fn pull_reviews_list(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
    ) -> Result<Vec<GhReviewApi>> {
        self.pull_reviews_list_response.clone()
    }

    async fn commit_statuses_update(
        &self,
        owner: &str,
        name: &str,
        git_ref: &str,
        status: StatusState,
        title: &str,
        body: &str,
    ) -> Result<()> {
        self.commit_status_update_response.clone()
    }

    async fn gif_search(&self, api_key: &str, search: &str) -> Result<GifResponse> {
        self.gif_search_response.clone()
    }
}