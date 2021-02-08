//! Utilities.

use github_scbot_core::Config;
use github_scbot_crypto::{create_jwt, now};
use octocrab::{Octocrab, OctocrabBuilder};
use serde::{Deserialize, Serialize};

use crate::{APIError, Result};

#[derive(Debug, Serialize, Deserialize)]
struct JwtClaims {
    iat: u64,
    exp: u64,
    iss: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct InstallationTokenResponse {
    token: String,
    expires_at: String,
}

/// Get an authenticated GitHub client.
///
/// # Arguments
///
/// * `config` - Bot configuration
pub async fn get_client(config: &Config) -> Result<Octocrab> {
    let client = get_client_builder(config).await?.build()?;

    Ok(client)
}

/// Get an authenticated GitHub client builder.
///
/// # Arguments
///
/// * `config` - Bot configuration
pub async fn get_client_builder(config: &Config) -> Result<OctocrabBuilder> {
    let token = get_authentication_credentials(config).await?;
    Ok(Octocrab::builder().personal_token(token))
}

pub(crate) fn is_client_enabled(config: &Config) -> bool {
    !config.api_disable_client
}

async fn get_authentication_credentials(config: &Config) -> Result<String> {
    if config.github_api_token.is_empty() {
        create_installation_access_token(config).await
    } else {
        Ok(config.github_api_token.clone())
    }
}

fn create_app_token(config: &Config) -> Result<String> {
    // GitHub App authentication documentation
    // https://docs.github.com/en/developers/apps/authenticating-with-github-apps#authenticating-as-a-github-app

    let now_ts = now();
    let claims = JwtClaims {
        // Issued at time
        iat: now_ts,
        // Expiration time, 1 minute
        exp: now_ts + 60,
        // GitHub App Identifier
        iss: config.github_app_id,
    };

    create_jwt(&config.github_app_private_key, &claims).map_err(Into::into)
}

async fn create_installation_access_token(config: &Config) -> Result<String> {
    let auth_token = create_app_token(config)?;
    let client = Octocrab::builder().personal_token(auth_token).build()?;
    let installation_id = config.github_app_installation_id;

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
            .map_err(|e| APIError::GitHubError(format!("Bad response: {}", e)))?;
        Ok(inst_resp.token)
    } else {
        Err(APIError::GitHubError(format!(
            "Bad status code: {}",
            status
        )))
    }
}
