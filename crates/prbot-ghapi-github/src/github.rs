//! GitHub adapter

use std::{future::Future, time::Duration};

use async_trait::async_trait;
use backoff::ExponentialBackoffBuilder;
use prbot_config::Config;
use prbot_ghapi_interface::{
    gif::GifResponse,
    review::GhReviewApi,
    types::{
        GhCheckRun, GhCommitStatus, GhCommitStatusState, GhMergeStrategy, GhPullRequest,
        GhReactionType, GhUserPermission,
    },
    ApiService, Result,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::{
    auth::{build_github_url, get_anonymous_client_builder, get_authenticated_client_builder},
    errors::GitHubError,
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

    async fn get_client(&self) -> Result<Client, GitHubError> {
        get_authenticated_client_builder(&self.config, self)
            .await?
            .build()
            .map_err(|e| GitHubError::HttpError { source: e })
    }

    fn build_url(&self, path: String) -> String {
        build_github_url(&self.config, path)
    }

    async fn call_with_retry<F, Fut, T>(&self, f: F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<T, GitHubError>>,
    {
        let conf = ExponentialBackoffBuilder::default()
            .with_max_elapsed_time(Some(Duration::from_secs(10)))
            .build();

        backoff::future::retry(conf, || async {
            f().await.map_err(|e| match e {
                GitHubError::HttpError { .. } => backoff::Error::transient(e),
                _ => backoff::Error::permanent(e),
            })
        })
        .await
        .map_err(Into::into)
    }
}

#[async_trait]
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

        self.call_with_retry(|| async move {
            Ok(self
                .get_client()
                .await?
                .get(&self.build_url(format!(
                    "/repos/{owner}/{name}/issues/{issue_number}/labels"
                )))
                .query(&[("per_page", 100)])
                .send()
                .await?
                .error_for_status()?
                .json::<Vec<Label>>()
                .await?
                .into_iter()
                .map(|x| x.name)
                .collect())
        })
        .await
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

        self.call_with_retry(|| async move {
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
        })
        .await
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

        self.call_with_retry(|| async move {
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
        })
        .await
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

        self.call_with_retry(|| async move {
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
        })
        .await
    }

    #[tracing::instrument(skip(self))]
    async fn check_runs_list(
        &self,
        owner: &str,
        name: &str,
        git_ref: &str,
    ) -> Result<Vec<GhCheckRun>> {
        #[derive(Deserialize)]
        struct Response {
            check_runs: Vec<GhCheckRun>,
        }

        let mut responses = vec![];
        let mut curr_page = 1;
        let max_per_page = 100;

        loop {
            debug!(current = curr_page, message = "Fetching check runs page");

            let results: Vec<GhCheckRun> = self
                .call_with_retry(|| async move {
                    let response = self
                        .get_client()
                        .await?
                        .get(&self.build_url(format!(
                            "/repos/{owner}/{name}/commits/{git_ref}/check-runs"
                        )))
                        .query(&[
                            ("per_page", max_per_page.to_string()),
                            ("page", curr_page.to_string()),
                            ("filter", "latest".into()),
                        ])
                        .send()
                        .await?
                        .error_for_status()?
                        .json::<Response>()
                        .await?;

                    Ok(response.check_runs)
                })
                .await?;

            match results.len() {
                0 => break,
                n if n == max_per_page => {
                    responses.extend(results);
                    curr_page += 1;
                }
                _ => {
                    responses.extend(results);
                    break;
                }
            }
        }

        Ok(responses)
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

        self.call_with_retry(|| async move {
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
        })
        .await
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

        self.call_with_retry(|| async move {
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
        })
        .await
    }

    #[tracing::instrument(skip(self))]
    async fn comments_delete(&self, owner: &str, name: &str, comment_id: u64) -> Result<()> {
        self.call_with_retry(|| async move {
            self.get_client()
                .await?
                .delete(&self.build_url(format!(
                    "/repos/{owner}/{name}/issues/comments/{comment_id}"
                )))
                .send()
                .await?
                .error_for_status()?;

            Ok(())
        })
        .await
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

        self.call_with_retry(|| async move {
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
        })
        .await
    }

    #[tracing::instrument(skip(self))]
    async fn pulls_get(&self, owner: &str, name: &str, number: u64) -> Result<GhPullRequest> {
        self.call_with_retry(|| async move {
            Ok(self
                .get_client()
                .await?
                .get(&self.build_url(format!("/repos/{owner}/{name}/pulls/{number}")))
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?)
        })
        .await
    }

    #[tracing::instrument(skip(self))]
    async fn pulls_merge(
        &self,
        owner: &str,
        name: &str,
        number: u64,
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

        self.call_with_retry(|| async move {
            self.get_client()
                .await?
                .put(&self.build_url(format!("/repos/{owner}/{name}/pulls/{number}/merge")))
                .json(&Request {
                    commit_title,
                    commit_message,
                    merge_method: merge_strategy.to_string(),
                })
                .send()
                .await?
                .error_for_status()
                .map_err(|_| GitHubError::MergeError {
                    pr_number: number,
                    repository_path: format!("{owner}/{name}"),
                })?;

            Ok(())
        })
        .await
    }

    #[tracing::instrument(skip(self))]
    async fn pull_reviewer_requests_add(
        &self,
        owner: &str,
        name: &str,
        number: u64,
        reviewers: &[String],
    ) -> Result<()> {
        #[derive(Serialize)]
        struct Request<'a> {
            reviewers: &'a [String],
        }

        self.call_with_retry(|| async move {
            self.get_client()
                .await?
                .post(&self.build_url(format!(
                    "/repos/{owner}/{name}/pulls/{number}/requested_reviewers"
                )))
                .json(&Request { reviewers })
                .send()
                .await?
                .error_for_status()?;

            Ok(())
        })
        .await
    }

    #[tracing::instrument(skip(self))]
    async fn pull_reviewer_requests_remove(
        &self,
        owner: &str,
        name: &str,
        number: u64,
        reviewers: &[String],
    ) -> Result<()> {
        #[derive(Serialize)]
        struct Request<'a> {
            reviewers: &'a [String],
        }

        self.call_with_retry(|| async move {
            self.get_client()
                .await?
                .delete(&self.build_url(format!(
                    "/repos/{owner}/{name}/pulls/{number}/requested_reviewers"
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
        })
        .await
    }

    #[tracing::instrument(skip(self))]
    async fn pull_reviews_list(
        &self,
        owner: &str,
        name: &str,
        number: u64,
    ) -> Result<Vec<GhReviewApi>> {
        let mut responses = vec![];
        let mut curr_page = 1;
        let max_per_page = 100;

        loop {
            debug!(current = curr_page, message = "Fetching review page");

            let results: Vec<GhReviewApi> = self
                .call_with_retry(|| async move {
                    self.get_client()
                        .await?
                        .get(
                            &self
                                .build_url(format!("/repos/{owner}/{name}/pulls/{number}/reviews")),
                        )
                        .query(&[("per_page", max_per_page), ("page", curr_page)])
                        .send()
                        .await?
                        .error_for_status()?
                        .json()
                        .await
                        .map_err(Into::into)
                })
                .await?;

            match results.len() {
                0 => {
                    break;
                }
                n if n == max_per_page => {
                    responses.extend(results);
                    curr_page += 1;
                }
                _ => {
                    responses.extend(results);
                    break;
                }
            }
        }

        Ok(responses)
    }

    #[tracing::instrument(skip(self))]
    async fn commit_statuses_combined(
        &self,
        owner: &str,
        name: &str,
        git_ref: &str,
    ) -> Result<GhCommitStatus> {
        let data = self
            .call_with_retry(|| async move {
                self.get_client()
                    .await?
                    .get(&self.build_url(format!("/repos/{owner}/{name}/commits/{git_ref}/status")))
                    .send()
                    .await?
                    .error_for_status()?
                    .json()
                    .await
                    .map_err(Into::into)
            })
            .await?;

        Ok(data)
    }

    #[tracing::instrument(skip(self))]
    async fn commit_statuses_update(
        &self,
        owner: &str,
        name: &str,
        git_ref: &str,
        status: GhCommitStatusState,
        title: &str,
        body: &str,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct Request<'a> {
            state: &'a str,
            description: String,
            context: &'a str,
        }

        self.call_with_retry(|| async move {
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
        })
        .await
    }

    #[tracing::instrument(skip(self))]
    async fn gif_search(&self, api_key: &str, search: &str) -> Result<GifResponse> {
        self.call_with_retry(|| async move {
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
                .map_err(Into::into)
        })
        .await
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

        self.call_with_retry(|| async move {
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
        })
        .await
    }
}
