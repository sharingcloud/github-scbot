//! GitHub adapter

use github_scbot_conf::Config;
use github_scbot_libs::{
    async_trait::async_trait, http, octocrab, octocrab::Octocrab, reqwest, serde_json,
    tracing::error,
};
use github_scbot_types::{
    checks::GhCheckSuite,
    common::GhUserPermission,
    issues::GhReactionType,
    pulls::{GhMergeStrategy, GhPullRequest},
    status::StatusState,
};
use serde::Deserialize;

use crate::{
    adapter::{GhReviewApi, GifResponse, IAPIAdapter},
    auth::get_client_builder,
    ApiError, Result,
};

const MAX_STATUS_DESCRIPTION_LEN: usize = 139;
const GIF_API_URL: &str = "https://g.tenor.com/v1";

/// GitHub API adapter implementation.
#[derive(Clone)]
pub struct GithubAPIAdapter {
    config: Config,
}

impl GithubAPIAdapter {
    /// Creates new GitHub API adapter.
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    async fn get_client(&self) -> Result<Octocrab> {
        get_client_builder(&self.config, self)
            .await?
            .add_preview("squirrel-girl")
            .build()
            .map_err(ApiError::from)
    }
}

#[async_trait]
impl IAPIAdapter for GithubAPIAdapter {
    async fn issue_labels_list(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
    ) -> Result<Vec<String>> {
        Ok(self
            .get_client()
            .await?
            .issues(owner, name)
            .list_labels_for_issue(issue_number)
            .send()
            .await?
            .take_items()
            .into_iter()
            .map(|x| x.name)
            .collect())
    }

    async fn issue_labels_replace_all(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
        labels: &[String],
    ) -> Result<()> {
        self.get_client()
            .await?
            .issues(owner, name)
            .replace_all_labels(issue_number, labels)
            .await
            .map_err(ApiError::from)
            .map(|_| ())
    }

    async fn user_permissions_get(
        &self,
        owner: &str,
        name: &str,
        username: &str,
    ) -> Result<GhUserPermission> {
        #[derive(Deserialize)]
        struct PermissionResponse {
            permission: GhUserPermission,
        }

        let output: PermissionResponse = self
            .get_client()
            .await?
            .get(
                format!(
                    "/repos/{owner}/{repo}/collaborators/{username}/permission",
                    owner = owner,
                    repo = name,
                    username = username
                ),
                None::<&()>,
            )
            .await?;

        Ok(output.permission)
    }

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

        let client = self.get_client().await?;
        let response: Response = client
            ._get(
                client.absolute_url(format!(
                    "/repos/{owner}/{name}/commits/{git_ref}/check-suites",
                    owner = owner,
                    name = name,
                    git_ref = git_ref
                ))?,
                None::<&()>,
            )
            .await?
            .json()
            .await?;

        Ok(response.check_suites)
    }

    async fn comments_post(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
        body: &str,
    ) -> Result<u64> {
        Ok(self
            .get_client()
            .await?
            .issues(owner, name)
            .create_comment(issue_number, body)
            .await?
            .id)
    }

    async fn comments_update(
        &self,
        owner: &str,
        name: &str,
        comment_id: u64,
        body: &str,
    ) -> Result<u64> {
        Ok(self
            .get_client()
            .await?
            .issues(owner, name)
            .update_comment(comment_id, body)
            .await?
            .id)
    }

    async fn comments_delete(&self, owner: &str, name: &str, comment_id: u64) -> Result<()> {
        self.get_client()
            .await?
            .issues(owner, name)
            .delete_comment(comment_id)
            .await?;

        Ok(())
    }

    async fn comment_reactions_add(
        &self,
        owner: &str,
        name: &str,
        comment_id: u64,
        reaction_type: GhReactionType,
    ) -> Result<()> {
        let body = serde_json::json!({
            "content": reaction_type.to_str()
        });

        let client = self.get_client().await?;
        let data = client
            ._post(
                client.absolute_url(format!(
                    "/repos/{}/{}/issues/comments/{}/reactions",
                    owner, name, comment_id
                ))?,
                Some(&body),
            )
            .await?;

        if data.status() != 201 {
            error!(
                status_code = %data.status(),
                message = "Could not add reaction to comment",
            );
        }

        Ok(())
    }

    async fn pulls_get(&self, owner: &str, name: &str, issue_number: u64) -> Result<GhPullRequest> {
        let pull: GhPullRequest = self
            .get_client()
            .await?
            .get(
                format!(
                    "/repos/{owner}/{name}/pulls/{pr_number}",
                    owner = owner,
                    name = name,
                    pr_number = issue_number
                ),
                None::<&()>,
            )
            .await?;

        Ok(pull)
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
        let body = serde_json::json!({
            "commit_title": commit_title,
            "commit_message": commit_message,
            "merge_method": merge_strategy.to_string()
        });

        let client = self.get_client().await?;

        let response = client
            ._put(
                client.absolute_url(format!(
                    "/repos/{}/{}/pulls/{}/merge",
                    owner, name, issue_number
                ))?,
                Some(&body),
            )
            .await?;

        let code: u16 = response.status().into();
        return match code {
            403 => Err(ApiError::MergeError("Forbidden".to_string())),
            404 => Err(ApiError::MergeError("Not found".to_string())),
            405 => Err(ApiError::MergeError("Not mergeable".to_string())),
            409 => Err(ApiError::MergeError("Conflicts".to_string())),
            _ => Ok(()),
        };
    }

    async fn pull_reviewer_requests_add(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
        reviewers: &[String],
    ) -> Result<()> {
        let body = serde_json::json!({ "reviewers": reviewers });
        let client = self.get_client().await?;

        client
            ._post(
                client.absolute_url(format!(
                    "/repos/{}/{}/pulls/{}/requested_reviewers",
                    owner, name, issue_number
                ))?,
                Some(&body),
            )
            .await?;

        Ok(())
    }

    async fn pull_reviewer_requests_remove(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
        reviewers: &[String],
    ) -> Result<()> {
        let body = serde_json::json!({ "reviewers": reviewers });
        let client = self.get_client().await?;
        let url = client.absolute_url(format!(
            "/repos/{}/{}/pulls/{}/requested_reviewers",
            owner, name, issue_number
        ))?;
        let builder = client
            .request_builder(&url.into_string(), http::Method::DELETE)
            .json(&body)
            .header(http::header::ACCEPT, octocrab::format_media_type("json"));

        let response = client.execute(builder).await?;
        if response.status() != 200 {
            error!(
                reviewers = ?reviewers,
                repository_path = %format!("{}/{}", owner, name),
                status_code = %response.status(),
                message = "Could not remove reviewers",
            );
        }

        Ok(())
    }

    async fn pull_reviews_list(
        &self,
        owner: &str,
        name: &str,
        issue_number: u64,
    ) -> Result<Vec<GhReviewApi>> {
        let data: Vec<GhReviewApi> = self
            .get_client()
            .await?
            .get(
                format!(
                    "/repos/{owner}/{name}/pulls/{pr_number}/reviews",
                    owner = owner,
                    name = name,
                    pr_number = issue_number
                ),
                None::<&()>,
            )
            .await?;

        Ok(data)
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
        let body = serde_json::json!({
            "state": status.to_str(),
            "description": body.chars().take(MAX_STATUS_DESCRIPTION_LEN).collect::<String>(),
            "context": title
        });

        let client = self.get_client().await?;

        client
            ._post(
                client.absolute_url(format!("/repos/{}/{}/statuses/{}", owner, name, git_ref))?,
                Some(&body),
            )
            .await?;

        Ok(())
    }

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
        #[derive(Debug, Deserialize)]
        struct InstallationTokenResponse {
            token: String,
            expires_at: String,
        }

        let client = Octocrab::builder()
            .personal_token(auth_token.into())
            .build()?;

        let response = client
            ._post(
                client.absolute_url(&format!(
                    "/app/installations/{}/access_tokens",
                    installation_id
                ))?,
                None::<&()>,
            )
            .await?;

        let status = response.status();
        if status == 201 {
            let inst_resp: InstallationTokenResponse = response
                .json()
                .await
                .map_err(|e| ApiError::GitHubError(format!("Bad response: {}", e)))?;
            Ok(inst_resp.token)
        } else {
            Err(ApiError::GitHubError(format!(
                "Bad status code: {}",
                status
            )))
        }
    }
}
