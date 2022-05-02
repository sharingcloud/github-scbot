//! GitHub adapter

use async_trait::async_trait;
use github_scbot_conf::Config;
use github_scbot_types::{
    checks::GhCheckSuite,
    common::GhUserPermission,
    issues::GhReactionType,
    pulls::{GhMergeStrategy, GhPullRequest},
    status::StatusState,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{
    adapter::{ApiService, GhReviewApi, GifResponse},
    auth::{build_github_url, get_anonymous_client_builder, get_authenticated_client_builder},
    ApiError, Result,
};

const MAX_STATUS_DESCRIPTION_LEN: usize = 139;
const GIF_API_URL: &str = "https://g.tenor.com/v1";

/// GitHub API adapter implementation.
#[derive(Clone)]
pub struct GithubApiService {
    config: Config,
}

impl GithubApiService {
    /// Creates new GitHub API adapter.
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    async fn get_client(&self) -> Result<Client> {
        get_authenticated_client_builder(&self.config, self)
            .await?
            .build()
            .map_err(ApiError::from)
    }

    fn build_url(&self, path: String) -> String {
        build_github_url(&self.config, path)
    }
}

#[async_trait(?Send)]
impl ApiService for GithubApiService {
    #[tracing::instrument(skip(self), ret)]
    async fn issue_labels_list(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
    ) -> Result<Vec<String>> {
        #[derive(Deserialize)]
        struct Label {
            name: String,
        }

        Ok(self
            .get_client()
            .await?
            .get(&self.build_url(format!(
                "/repos/{owner}/{name}/issues/{issue_number}/labels"
            )))
            .send()
            .await?
            .error_for_status()?
            .json::<Vec<Label>>()
            .await?
            .into_iter()
            .map(|x| x.name)
            .collect())
    }

    #[tracing::instrument(skip(self))]
    async fn issue_labels_replace_all(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
        labels: &[String],
    ) -> Result<()> {
        #[derive(Serialize)]
        struct Request<'a> {
            labels: &'a [String],
        }

        self.get_client()
            .await?
            .put(&self.build_url(format!(
                "/repos/{owner}/{name}/issues/{issue_number}/labels"
            )))
            .json(&Request { labels })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn issue_labels_add(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
        labels: &[String],
    ) -> Result<()> {
        #[derive(Serialize)]
        struct Request<'a> {
            labels: &'a [String],
        }

        self.get_client()
            .await?
            .post(&self.build_url(format!(
                "/repos/{owner}/{name}/issues/{issue_number}/labels"
            )))
            .json(&Request { labels })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    #[tracing::instrument(skip(self), ret)]
    async fn user_permissions_get(
        &self,
        owner: &str,
        name: &str,
        username: &str,
    ) -> Result<GhUserPermission> {
        #[derive(Deserialize)]
        struct Response {
            permission: GhUserPermission,
        }

        let response = self
            .get_client()
            .await?
            .get(&self.build_url(format!(
                "/repos/{owner}/{name}/collaborators/{username}/permission"
            )))
            .send()
            .await?
            .error_for_status()?
            .json::<Response>()
            .await?;

        Ok(response.permission)
    }

    #[tracing::instrument(skip(self), ret)]
    async fn check_suites_list(
        &self,
        owner: &str,
        name: &str,
        git_ref: &str,
    ) -> Result<Vec<GhCheckSuite>> {
        #[derive(Deserialize)]
        struct Response {
            check_suites: Vec<GhCheckSuite>,
        }

        let response = self
            .get_client()
            .await?
            .get(&self.build_url(format!(
                "/repos/{owner}/{name}/commits/{git_ref}/check-suites"
            )))
            .send()
            .await?
            .error_for_status()?
            .json::<Response>()
            .await?;

        Ok(response.check_suites)
    }

    #[tracing::instrument(skip(self), ret)]
    async fn comments_post(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
        body: &str,
    ) -> Result<u64> {
        #[derive(Serialize)]
        struct Request<'a> {
            body: &'a str,
        }

        #[derive(Deserialize)]
        struct Response {
            id: u64,
        }

        Ok(self
            .get_client()
            .await?
            .post(&self.build_url(format!(
                "/repos/{owner}/{name}/issues/{issue_number}/comments"
            )))
            .json(&Request { body })
            .send()
            .await?
            .error_for_status()?
            .json::<Response>()
            .await?
            .id)
    }

    #[tracing::instrument(skip(self), ret)]
    async fn comments_update(
        &self,
        owner: &str,
        name: &str,
        comment_id: u64,
        body: &str,
    ) -> Result<u64> {
        #[derive(Serialize)]
        struct Request<'a> {
            body: &'a str,
        }

        #[derive(Deserialize)]
        struct Response {
            id: u64,
        }

        Ok(self
            .get_client()
            .await?
            .patch(&self.build_url(format!(
                "/repos/{owner}/{name}/issues/comments/{comment_id}"
            )))
            .json(&Request { body })
            .send()
            .await?
            .error_for_status()?
            .json::<Response>()
            .await?
            .id)
    }

    #[tracing::instrument(skip(self))]
    async fn comments_delete(&self, owner: &str, name: &str, comment_id: u64) -> Result<()> {
        self.get_client()
            .await?
            .delete(&self.build_url(format!(
                "/repos/{owner}/{name}/issues/comments/{comment_id}"
            )))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn comment_reactions_add(
        &self,
        owner: &str,
        name: &str,
        comment_id: u64,
        reaction_type: GhReactionType,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct Request<'a> {
            content: &'a str,
        }

        self.get_client()
            .await?
            .post(&self.build_url(format!(
                "/repos/{owner}/{name}/issues/comments/{comment_id}/reactions"
            )))
            .json(&Request {
                content: reaction_type.to_str(),
            })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    #[tracing::instrument(skip(self), ret)]
    async fn pulls_get(&self, owner: &str, name: &str, issue_number: u64) -> Result<GhPullRequest> {
        Ok(self
            .get_client()
            .await?
            .get(&self.build_url(format!("/repos/{owner}/{name}/pulls/{issue_number}")))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    #[tracing::instrument(skip(self))]
    async fn pulls_merge(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
        commit_title: &str,
        commit_message: &str,
        merge_strategy: GhMergeStrategy,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct Request<'a> {
            commit_title: &'a str,
            commit_message: &'a str,
            merge_method: String,
        }

        self.get_client()
            .await?
            .put(&self.build_url(format!("/repos/{owner}/{name}/pulls/{issue_number}/merge")))
            .json(&Request {
                commit_title,
                commit_message,
                merge_method: merge_strategy.to_string(),
            })
            .send()
            .await?
            .error_for_status()
            .map_err(|e| ApiError::MergeError(e.to_string()))?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn pull_reviewer_requests_add(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
        reviewers: &[String],
    ) -> Result<()> {
        #[derive(Serialize)]
        struct Request<'a> {
            reviewers: &'a [String],
        }

        self.get_client()
            .await?
            .post(&self.build_url(format!(
                "/repos/{owner}/{name}/pulls/{issue_number}/requested_reviewers"
            )))
            .json(&Request { reviewers })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn pull_reviewer_requests_remove(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
        reviewers: &[String],
    ) -> Result<()> {
        #[derive(Serialize)]
        struct Request<'a> {
            reviewers: &'a [String],
        }

        self.get_client()
            .await?
            .delete(&self.build_url(format!(
                "/repos/{owner}/{name}/pulls/{issue_number}/requested_reviewers"
            )))
            .json(&Request { reviewers })
            .header(
                http::header::ACCEPT,
                http::header::HeaderValue::from_static("application/vnd.github.v3+json"),
            )
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    #[tracing::instrument(skip(self), ret)]
    async fn pull_reviews_list(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
    ) -> Result<Vec<GhReviewApi>> {
        Ok(self
            .get_client()
            .await?
            .get(&self.build_url(format!(
                "/repos/{owner}/{name}/pulls/{issue_number}/reviews"
            )))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    #[tracing::instrument(skip(self))]
    async fn commit_statuses_update(
        &self,
        owner: &str,
        name: &str,
        git_ref: &str,
        status: StatusState,
        title: &str,
        body: &str,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct Request<'a> {
            state: &'a str,
            description: String,
            context: &'a str,
        }

        self.get_client()
            .await?
            .post(&self.build_url(format!("/repos/{owner}/{name}/statuses/{git_ref}")))
            .json(&Request {
                state: status.to_str(),
                context: title,
                description: body
                    .chars()
                    .take(MAX_STATUS_DESCRIPTION_LEN)
                    .collect::<String>(),
            })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    #[tracing::instrument(skip(self), ret)]
    async fn gif_search(&self, api_key: &str, search: &str) -> Result<GifResponse> {
        let client = reqwest::Client::new();
        client
            .get(&format!("{}/search", GIF_API_URL))
            .query(&[
                ("q", search),
                ("key", api_key),
                ("limit", "3"),
                ("locale", "en_US"),
                ("contentfilter", "low"),
                ("media_filter", "basic"),
                ("ar_range", "all"),
            ])
            .send()
            .await?
            .json()
            .await
            .map_err(ApiError::from)
    }

    async fn installations_create_token(
        &self,
        auth_token: &str,
        installation_id: u64,
    ) -> Result<String> {
        #[derive(Deserialize)]
        struct Response {
            token: String,
        }

        Ok(get_anonymous_client_builder(&self.config)?
            .build()?
            .post(&self.build_url(format!(
                "/app/installations/{installation_id}/access_tokens"
            )))
            .bearer_auth(auth_token)
            .send()
            .await?
            .error_for_status()?
            .json::<Response>()
            .await?
            .token)
    }
}
