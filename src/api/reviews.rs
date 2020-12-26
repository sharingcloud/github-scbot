//! Reviews API module

use crate::database::models::{PullRequestModel, RepositoryModel};

use super::errors::Result;
use super::get_client;

pub async fn request_reviewers_for_pr(
    repo_model: &RepositoryModel,
    pr_model: &PullRequestModel,
    reviewers: &[String],
) -> Result<()> {
    if !cfg!(test) {
        let client = get_client().await?;
        let body = serde_json::json!({ "reviewers": reviewers });

        client
            ._post(
                client.absolute_url(format!(
                    "/repos/{}/{}/pulls/{}/requested_reviewers",
                    &repo_model.owner, &repo_model.name, pr_model.number
                ))?,
                Some(&body),
            )
            .await?;
    }

    Ok(())
}

pub async fn remove_reviewers_for_pr(
    repo_model: &RepositoryModel,
    pr_model: &PullRequestModel,
    reviewers: &[String],
) -> Result<()> {
    if !cfg!(test) {
        let client = get_client().await?;
        let body = serde_json::json!({ "reviewers": reviewers });

        client
            ._delete(
                client.absolute_url(format!(
                    "/repos/{}/{}/pulls/{}/requested_reviewers",
                    &repo_model.owner, &repo_model.name, pr_model.number
                ))?,
                Some(&body),
            )
            .await?;
    }

    Ok(())
}
