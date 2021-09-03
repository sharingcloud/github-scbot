//! Dummy adapter

use github_scbot_libs::async_trait::async_trait;
use github_scbot_types::{
    checks::GhCheckSuite,
    common::GhUserPermission,
    issues::GhReactionType,
    pulls::{GhMergeStrategy, GhPullRequest},
    status::StatusState,
};
use github_scbot_utils::Mock;

use super::{GhReviewApi, GifResponse, IAPIAdapter};
use crate::Result;

/// Dummy API adapter.
#[allow(missing_docs)]
pub struct DummyAPIAdapter {
    pub issue_labels_list_response: Mock<Result<Vec<String>>>,
    pub issue_labels_replace_all_response: Mock<Result<()>>,
    pub user_permissions_get_response: Mock<Result<GhUserPermission>>,
    pub check_suites_list_response: Mock<Result<Vec<GhCheckSuite>>>,
    pub comments_post_response: Mock<Result<u64>>,
    pub comments_update_response: Mock<Result<u64>>,
    pub comments_delete_response: Mock<Result<()>>,
    pub comment_reactions_add_response: Mock<Result<()>>,
    pub pulls_get_response: Mock<Result<GhPullRequest>>,
    pub pulls_merge_response: Mock<Result<()>>,
    pub pull_reviewer_requests_add_response: Mock<Result<()>>,
    pub pull_reviewer_requests_remove_response: Mock<Result<()>>,
    pub pull_reviews_list_response: Mock<Result<Vec<GhReviewApi>>>,
    pub commit_status_update_response: Mock<Result<()>>,
    pub gif_search_response: Mock<Result<GifResponse>>,
    pub installations_create_token_response: Mock<Result<String>>,
}

impl Default for DummyAPIAdapter {
    fn default() -> Self {
        Self {
            issue_labels_list_response: Mock::new(Ok(Vec::new())),
            issue_labels_replace_all_response: Mock::new(Ok(())),
            user_permissions_get_response: Mock::new(Ok(GhUserPermission::None)),
            check_suites_list_response: Mock::new(Ok(Vec::new())),
            comments_post_response: Mock::new(Ok(0)),
            comments_update_response: Mock::new(Ok(0)),
            comments_delete_response: Mock::new(Ok(())),
            comment_reactions_add_response: Mock::new(Ok(())),
            pulls_get_response: Mock::new(Ok(GhPullRequest::default())),
            pulls_merge_response: Mock::new(Ok(())),
            pull_reviewer_requests_add_response: Mock::new(Ok(())),
            pull_reviewer_requests_remove_response: Mock::new(Ok(())),
            pull_reviews_list_response: Mock::new(Ok(Vec::new())),
            commit_status_update_response: Mock::new(Ok(())),
            gif_search_response: Mock::new(Ok(GifResponse::default())),
            installations_create_token_response: Mock::new(Ok(String::new())),
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
        self.issue_labels_list_response.response()
    }

    async fn issue_labels_replace_all(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
        labels: &[String],
    ) -> Result<()> {
        self.issue_labels_replace_all_response.response()
    }

    async fn user_permissions_get(
        &self,
        owner: &str,
        name: &str,
        username: &str,
    ) -> Result<GhUserPermission> {
        self.user_permissions_get_response.response()
    }

    async fn check_suites_list(
        &self,
        owner: &str,
        name: &str,
        git_ref: &str,
    ) -> Result<Vec<GhCheckSuite>> {
        self.check_suites_list_response.response()
    }

    async fn comments_post(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
        body: &str,
    ) -> Result<u64> {
        self.comments_post_response.response()
    }

    async fn comments_update(
        &self,
        owner: &str,
        name: &str,
        comment_id: u64,
        body: &str,
    ) -> Result<u64> {
        self.comments_update_response.response()
    }

    async fn comments_delete(&self, owner: &str, name: &str, comment_id: u64) -> Result<()> {
        self.comments_delete_response.response()
    }

    async fn comment_reactions_add(
        &self,
        owner: &str,
        name: &str,
        comment_id: u64,
        reaction_type: GhReactionType,
    ) -> Result<()> {
        self.comment_reactions_add_response.response()
    }

    async fn pulls_get(&self, owner: &str, name: &str, issue_number: u64) -> Result<GhPullRequest> {
        self.pulls_get_response.response()
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
        self.pulls_merge_response.response()
    }

    async fn pull_reviewer_requests_add(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
        reviewers: &[String],
    ) -> Result<()> {
        self.pull_reviewer_requests_add_response.response()
    }

    async fn pull_reviewer_requests_remove(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
        reviewers: &[String],
    ) -> Result<()> {
        self.pull_reviewer_requests_remove_response.response()
    }

    async fn pull_reviews_list(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
    ) -> Result<Vec<GhReviewApi>> {
        self.pull_reviews_list_response.response()
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
        self.commit_status_update_response.response()
    }

    async fn gif_search(&self, api_key: &str, search: &str) -> Result<GifResponse> {
        self.gif_search_response.response()
    }

    async fn installations_create_token(
        &self,
        auth_token: &str,
        installation_id: u64,
    ) -> Result<String> {
        self.installations_create_token_response.response()
    }
}
