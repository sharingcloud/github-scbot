//! Checks API module.

use github_scbot_conf::Config;
use github_scbot_types::checks::GHCheckSuite;
use serde::Deserialize;

use crate::{
    utils::{get_client, is_client_enabled},
    APIError, Result,
};

/// List check-suites for Git Reference.
///
/// # Arguments
///
/// * `config` - Bot configuration
/// * `repository_owner` - Repository owner
/// * `repository_name` - Repository name
/// * `git_ref` - Git reference
pub async fn list_check_suites_for_git_ref(
    config: &Config,
    repository_owner: &str,
    repository_name: &str,
    git_ref: &str,
) -> Result<Vec<GHCheckSuite>> {
    if is_client_enabled(config) {
        #[derive(Deserialize)]
        struct Response {
            check_suites: Vec<GHCheckSuite>,
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
            .map_err(|e| APIError::GitHubError(e.to_string()))?;

        Ok(response.check_suites)
    } else {
        Ok(vec![])
    }
}
