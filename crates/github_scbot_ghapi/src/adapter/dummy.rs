//! Dummy adapter

use async_trait::async_trait;
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
#[allow(missing_docs, clippy::type_complexity)]
pub struct DummyAPIAdapter {
    pub issue_labels_list_response: Mock<(String, String, u64), Result<Vec<String>>>,
    pub issue_labels_replace_all_response: Mock<(String, String, u64, Vec<String>), Result<()>>,
    pub issue_labels_add_response: Mock<(String, String, u64, Vec<String>), Result<()>>,
    pub user_permissions_get_response: Mock<(String, String, String), Result<GhUserPermission>>,
    pub check_suites_list_response: Mock<(String, String, String), Result<Vec<GhCheckSuite>>>,
    pub comments_post_response: Mock<(String, String, u64, String), Result<u64>>,
    pub comments_update_response: Mock<(String, String, u64, String), Result<u64>>,
    pub comments_delete_response: Mock<(String, String, u64), Result<()>>,
    pub comment_reactions_add_response: Mock<(String, String, u64, GhReactionType), Result<()>>,
    pub pulls_get_response: Mock<(String, String, u64), Result<GhPullRequest>>,
    pub pulls_merge_response:
        Mock<(String, String, u64, String, String, GhMergeStrategy), Result<()>>,
    pub pull_reviewer_requests_add_response: Mock<(String, String, u64, Vec<String>), Result<()>>,
    pub pull_reviewer_requests_remove_response:
        Mock<(String, String, u64, Vec<String>), Result<()>>,
    pub pull_reviews_list_response: Mock<(String, String, u64), Result<Vec<GhReviewApi>>>,
    pub commit_status_update_response:
        Mock<(String, String, String, StatusState, String, String), Result<()>>,
    pub gif_search_response: Mock<(String, String), Result<GifResponse>>,
    pub installations_create_token_response: Mock<(String, u64), Result<String>>,
}

impl Default for DummyAPIAdapter {
    fn default() -> Self {
        Self {
            issue_labels_list_response: Mock::new(Box::new(|_| Ok(Vec::new()))),
            issue_labels_replace_all_response: Mock::new(Box::new(|_| Ok(()))),
            issue_labels_add_response: Mock::new(Box::new(|_| Ok(()))),
            user_permissions_get_response: Mock::new(Box::new(|_| Ok(GhUserPermission::None))),
            check_suites_list_response: Mock::new(Box::new(|_| Ok(Vec::new()))),
            comments_post_response: Mock::new(Box::new(|_| Ok(0))),
            comments_update_response: Mock::new(Box::new(|_| Ok(0))),
            comments_delete_response: Mock::new(Box::new(|_| Ok(()))),
            comment_reactions_add_response: Mock::new(Box::new(|_| Ok(()))),
            pulls_get_response: Mock::new(Box::new(|_| Ok(GhPullRequest::default()))),
            pulls_merge_response: Mock::new(Box::new(|_| Ok(()))),
            pull_reviewer_requests_add_response: Mock::new(Box::new(|_| Ok(()))),
            pull_reviewer_requests_remove_response: Mock::new(Box::new(|_| Ok(()))),
            pull_reviews_list_response: Mock::new(Box::new(|_| Ok(Vec::new()))),
            commit_status_update_response: Mock::new(Box::new(|_| Ok(()))),
            gif_search_response: Mock::new(Box::new(|_| Ok(GifResponse::default()))),
            installations_create_token_response: Mock::new(Box::new(|_| Ok(String::new()))),
        }
    }
}

impl DummyAPIAdapter {
    /// Creates new dummy API adapter.
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait(?Send)]
#[allow(unused_variables)]
impl IAPIAdapter for DummyAPIAdapter {
    async fn issue_labels_list(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
    ) -> Result<Vec<String>> {
        self.issue_labels_list_response
            .call((owner.to_owned(), name.to_owned(), issue_number))
    }

    async fn issue_labels_replace_all(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
        labels: &[String],
    ) -> Result<()> {
        self.issue_labels_replace_all_response.call((
            owner.to_owned(),
            name.to_owned(),
            issue_number,
            labels.to_owned(),
        ))
    }

    async fn issue_labels_add(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
        labels: &[String],
    ) -> Result<()> {
        self.issue_labels_add_response.call((
            owner.to_owned(),
            name.to_owned(),
            issue_number,
            labels.to_owned(),
        ))
    }

    async fn user_permissions_get(
        &self,
        owner: &str,
        name: &str,
        username: &str,
    ) -> Result<GhUserPermission> {
        self.user_permissions_get_response.call((
            owner.to_owned(),
            name.to_owned(),
            username.to_owned(),
        ))
    }

    async fn check_suites_list(
        &self,
        owner: &str,
        name: &str,
        git_ref: &str,
    ) -> Result<Vec<GhCheckSuite>> {
        self.check_suites_list_response.call((
            owner.to_owned(),
            name.to_owned(),
            git_ref.to_owned(),
        ))
    }

    async fn comments_post(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
        body: &str,
    ) -> Result<u64> {
        self.comments_post_response.call((
            owner.to_owned(),
            name.to_owned(),
            issue_number,
            body.to_owned(),
        ))
    }

    async fn comments_update(
        &self,
        owner: &str,
        name: &str,
        comment_id: u64,
        body: &str,
    ) -> Result<u64> {
        self.comments_update_response.call((
            owner.to_owned(),
            name.to_owned(),
            comment_id,
            body.to_owned(),
        ))
    }

    async fn comments_delete(&self, owner: &str, name: &str, comment_id: u64) -> Result<()> {
        self.comments_delete_response
            .call((owner.to_owned(), name.to_owned(), comment_id))
    }

    async fn comment_reactions_add(
        &self,
        owner: &str,
        name: &str,
        comment_id: u64,
        reaction_type: GhReactionType,
    ) -> Result<()> {
        self.comment_reactions_add_response.call((
            owner.to_owned(),
            name.to_owned(),
            comment_id,
            reaction_type,
        ))
    }

    async fn pulls_get(&self, owner: &str, name: &str, issue_number: u64) -> Result<GhPullRequest> {
        self.pulls_get_response
            .call((owner.to_owned(), name.to_owned(), issue_number))
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
        self.pulls_merge_response.call((
            owner.to_owned(),
            name.to_owned(),
            issue_number,
            commit_title.to_owned(),
            commit_message.to_owned(),
            merge_strategy,
        ))
    }

    async fn pull_reviewer_requests_add(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
        reviewers: &[String],
    ) -> Result<()> {
        self.pull_reviewer_requests_add_response.call((
            owner.to_owned(),
            name.to_owned(),
            issue_number,
            reviewers.to_owned(),
        ))
    }

    async fn pull_reviewer_requests_remove(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
        reviewers: &[String],
    ) -> Result<()> {
        self.pull_reviewer_requests_remove_response.call((
            owner.to_owned(),
            name.to_owned(),
            issue_number,
            reviewers.to_owned(),
        ))
    }

    async fn pull_reviews_list(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
    ) -> Result<Vec<GhReviewApi>> {
        self.pull_reviews_list_response
            .call((owner.to_owned(), name.to_owned(), issue_number))
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
        self.commit_status_update_response.call((
            owner.to_owned(),
            name.to_owned(),
            git_ref.to_owned(),
            status,
            title.to_owned(),
            body.to_owned(),
        ))
    }

    async fn gif_search(&self, api_key: &str, search: &str) -> Result<GifResponse> {
        self.gif_search_response
            .call((api_key.to_owned(), search.to_owned()))
    }

    async fn installations_create_token(
        &self,
        auth_token: &str,
        installation_id: u64,
    ) -> Result<String> {
        self.installations_create_token_response
            .call((auth_token.to_owned(), installation_id))
    }
}
