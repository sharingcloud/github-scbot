//! Status API module

use super::errors::Result;
use super::get_client;
use crate::database::models::RepositoryModel;

const MAX_STATUS_DESCRIPTION_LEN: usize = 139;

#[derive(Debug, PartialEq, Eq)]
pub enum StatusState {
    Error,
    Failure,
    Pending,
    Success,
}

impl StatusState {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Failure => "failure",
            Self::Pending => "pending",
            Self::Success => "success",
        }
    }
}

pub async fn update_status_for_repo(
    repo: &RepositoryModel,
    commit_sha: &str,
    status: StatusState,
    title: &str,
    body: &str,
) -> Result<()> {
    if !cfg!(test) {
        let client = get_client().await?;
        let body = serde_json::json!({
            "state": status.as_str(),
            "description": body.chars().take(MAX_STATUS_DESCRIPTION_LEN).collect::<String>(),
            "context": title
        });

        client
            ._post(
                client.absolute_url(format!(
                    "/repos/{}/{}/statuses/{}",
                    &repo.owner, &repo.name, commit_sha
                ))?,
                Some(&body),
            )
            .await?;
    }

    Ok(())
}
