//! Authentication and authorization module.

use github_scbot_conf::Config;
use github_scbot_types::common::GHUserPermission;
use serde::Deserialize;

use crate::{
    utils::{get_client, is_client_enabled},
    Result,
};

#[derive(Deserialize)]
struct PermissionResponse {
    permission: GHUserPermission,
}

/// Get user permission on repository.
///
/// # Arguments
///
/// * `config` - Bot configuration
/// * `repository_owner` - Repository owner
/// * `repository_name` - Repository name
/// * `username` - Target username
pub async fn get_user_permission_on_repository(
    config: &Config,
    repository_owner: &str,
    repository_name: &str,
    username: &str,
) -> Result<GHUserPermission> {
    if is_client_enabled(config) {
        let client = get_client(config).await?;

        let output: PermissionResponse = client
            .get(
                format!(
                    "/repos/{owner}/{repo}/collaborators/{username}/permission",
                    owner = repository_owner,
                    repo = repository_name,
                    username = username
                ),
                None::<&()>,
            )
            .await?;

        return Ok(output.permission);
    }

    Ok(GHUserPermission::None)
}
