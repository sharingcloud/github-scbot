//! GitHub Api wrappers.

use async_trait::async_trait;
use prbot_config::Config;
use prbot_ghapi_github::GithubApiService;
use prbot_ghapi_interface::{
    gif::GifResponse,
    review::GhReviewApi,
    types::{
        GhCheckRun, GhCommitStatus, GhCommitStatusState, GhMergeStrategy, GhPullRequest,
        GhReactionType, GhUserPermission,
    },
    ApiService, Result,
};

use crate::metrics::{GITHUB_API_CALLS, TENOR_API_CALLS};

/// GitHub Api Service with metrics.
pub struct MetricsApiService {
    inner: GithubApiService,
}

impl MetricsApiService {
    /// Creates a new service.
    pub fn new(config: Config) -> Self {
        Self {
            inner: GithubApiService::new(config),
        }
    }
}

#[async_trait]
impl ApiService for MetricsApiService {
    async fn issue_labels_list(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
    ) -> Result<Vec<String>> {
        GITHUB_API_CALLS.inc();
        self.inner
            .issue_labels_list(owner, name, issue_number)
            .await
    }

    async fn issue_labels_replace_all(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
        labels: &[String],
    ) -> Result<()> {
        GITHUB_API_CALLS.inc();
        self.inner
            .issue_labels_replace_all(owner, name, issue_number, labels)
            .await
    }

    async fn issue_labels_add(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
        labels: &[String],
    ) -> Result<()> {
        GITHUB_API_CALLS.inc();
        self.inner
            .issue_labels_add(owner, name, issue_number, labels)
            .await
    }

    async fn user_permissions_get(
        &self,
        owner: &str,
        name: &str,
        username: &str,
    ) -> Result<GhUserPermission> {
        GITHUB_API_CALLS.inc();
        self.inner.user_permissions_get(owner, name, username).await
    }

    async fn check_runs_list(
        &self,
        owner: &str,
        name: &str,
        git_ref: &str,
    ) -> Result<Vec<GhCheckRun>> {
        GITHUB_API_CALLS.inc();
        self.inner.check_runs_list(owner, name, git_ref).await
    }

    async fn comments_post(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
        body: &str,
    ) -> Result<u64> {
        GITHUB_API_CALLS.inc();
        self.inner
            .comments_post(owner, name, issue_number, body)
            .await
    }

    async fn comments_update(
        &self,
        owner: &str,
        name: &str,
        comment_id: u64,
        body: &str,
    ) -> Result<u64> {
        GITHUB_API_CALLS.inc();
        self.inner
            .comments_update(owner, name, comment_id, body)
            .await
    }

    async fn comments_delete(&self, owner: &str, name: &str, comment_id: u64) -> Result<()> {
        GITHUB_API_CALLS.inc();
        self.inner.comments_delete(owner, name, comment_id).await
    }

    async fn comment_reactions_add(
        &self,
        owner: &str,
        name: &str,
        comment_id: u64,
        reaction_type: GhReactionType,
    ) -> Result<()> {
        GITHUB_API_CALLS.inc();
        self.inner
            .comment_reactions_add(owner, name, comment_id, reaction_type)
            .await
    }

    async fn pulls_get(&self, owner: &str, name: &str, number: u64) -> Result<GhPullRequest> {
        GITHUB_API_CALLS.inc();
        self.inner.pulls_get(owner, name, number).await
    }

    async fn pulls_merge(
        &self,
        owner: &str,
        name: &str,
        number: u64,
        commit_title: &str,
        commit_message: &str,
        merge_strategy: GhMergeStrategy,
    ) -> Result<()> {
        GITHUB_API_CALLS.inc();
        self.inner
            .pulls_merge(
                owner,
                name,
                number,
                commit_title,
                commit_message,
                merge_strategy,
            )
            .await
    }

    async fn pull_reviewer_requests_add(
        &self,
        owner: &str,
        name: &str,
        number: u64,
        reviewers: &[String],
    ) -> Result<()> {
        GITHUB_API_CALLS.inc();
        self.inner
            .pull_reviewer_requests_add(owner, name, number, reviewers)
            .await
    }

    async fn pull_reviewer_requests_remove(
        &self,
        owner: &str,
        name: &str,
        number: u64,
        reviewers: &[String],
    ) -> Result<()> {
        GITHUB_API_CALLS.inc();
        self.inner
            .pull_reviewer_requests_remove(owner, name, number, reviewers)
            .await
    }

    async fn pull_reviews_list(
        &self,
        owner: &str,
        name: &str,
        number: u64,
    ) -> Result<Vec<GhReviewApi>> {
        GITHUB_API_CALLS.inc();
        self.inner.pull_reviews_list(owner, name, number).await
    }

    async fn commit_statuses_combined(
        &self,
        owner: &str,
        name: &str,
        git_ref: &str,
    ) -> Result<GhCommitStatus> {
        GITHUB_API_CALLS.inc();
        self.inner
            .commit_statuses_combined(owner, name, git_ref)
            .await
    }

    async fn commit_statuses_update(
        &self,
        owner: &str,
        name: &str,
        git_ref: &str,
        status: GhCommitStatusState,
        title: &str,
        body: &str,
    ) -> Result<()> {
        GITHUB_API_CALLS.inc();
        self.inner
            .commit_statuses_update(owner, name, git_ref, status, title, body)
            .await
    }

    async fn gif_search(&self, api_key: &str, search: &str) -> Result<GifResponse> {
        TENOR_API_CALLS.inc();
        self.inner.gif_search(api_key, search).await
    }

    async fn installations_create_token(
        &self,
        auth_token: &str,
        installation_id: u64,
    ) -> Result<String> {
        GITHUB_API_CALLS.inc();
        self.inner
            .installations_create_token(auth_token, installation_id)
            .await
    }
}
