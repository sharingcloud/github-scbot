use async_trait::async_trait;

use crate::{
    gif::GifResponse,
    review::GhReviewApi,
    types::{
        GhCheckRun, GhCommitStatus, GhMergeStrategy, GhPullRequest, GhReactionType,
        GhUserPermission,
    },
    Result,
};

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
    /// List latest check runs from a repository.
    async fn check_runs_list(
        &self,
        owner: &str,
        name: &str,
        git_ref: &str,
    ) -> Result<Vec<GhCheckRun>>;
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
        status: GhCommitStatus,
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
