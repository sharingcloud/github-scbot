//! GitHub Api wrappers.

use async_trait::async_trait;
use github_scbot_core::config::Config;
use github_scbot_ghapi_impl::GithubApiService;
use github_scbot_ghapi_interface::{
    gif::GifResponse,
    review::GhReviewApi,
    types::{
        GhCheckSuite, GhCommitStatus, GhMergeStrategy, GhPullRequest, GhReactionType,
        GhUserPermission,
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

#[async_trait(?Send)]
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

    async fn check_suites_list(
        &self,
        owner: &str,
        name: &str,
        git_ref: &str,
    ) -> Result<Vec<GhCheckSuite>> {
        GITHUB_API_CALLS.inc();
        self.inner.check_suites_list(owner, name, git_ref).await
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

    async fn pulls_get(&self, owner: &str, name: &str, issue_number: u64) -> Result<GhPullRequest> {
        GITHUB_API_CALLS.inc();
        self.inner.pulls_get(owner, name, issue_number).await
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
        GITHUB_API_CALLS.inc();
        self.inner
            .pulls_merge(
                owner,
                name,
                issue_number,
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
        issue_number: u64,
        reviewers: &[String],
    ) -> Result<()> {
        GITHUB_API_CALLS.inc();
        self.inner
            .pull_reviewer_requests_add(owner, name, issue_number, reviewers)
            .await
    }

    async fn pull_reviewer_requests_remove(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
        reviewers: &[String],
    ) -> Result<()> {
        GITHUB_API_CALLS.inc();
        self.inner
            .pull_reviewer_requests_remove(owner, name, issue_number, reviewers)
            .await
    }

    async fn pull_reviews_list(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
    ) -> Result<Vec<GhReviewApi>> {
        GITHUB_API_CALLS.inc();
        self.inner
            .pull_reviews_list(owner, name, issue_number)
            .await
    }

    async fn commit_statuses_update(
        &self,
        owner: &str,
        name: &str,
        git_ref: &str,
        status: GhCommitStatus,
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
