//! Checks API module.

use github_scbot_conf::Config;
use github_scbot_types::checks::GhCheckSuite;
use serde::Deserialize;

use crate::{
    utils::{get_client, is_client_enabled},
    ApiError, Result,
};

/// List check-suites from Git Reference.
pub async fn list_check_suites_from_git_ref(
    config: &Config,
    repository_owner: &str,
    repository_name: &str,
    git_ref: &str,
) -> Result<Vec<GhCheckSuite>> {
    if is_client_enabled(config) {
        #[derive(Deserialize)]
        struct Response {
            check_suites: Vec<GhCheckSuite>,
        }

        let client = get_client(config).await?;
        let response: Response = client
            ._get(
                client.absolute_url(format!(
                    "/repos/{owner}/{name}/commits/{git_ref}/check-suites",
                    owner = repository_owner,
                    name = repository_name,
                    git_ref = git_ref
                ))?,
                None::<&()>,
            )
            .await?
            .json()
            .await
            .map_err(|e| ApiError::GitHubError(e.to_string()))?;

        Ok(response.check_suites)
    } else {
        Ok(vec![])
    }
}
