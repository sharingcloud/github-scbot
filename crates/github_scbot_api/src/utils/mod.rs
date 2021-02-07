//! Utilities.

use std::time::{SystemTime, UNIX_EPOCH};

use github_scbot_core::Config;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use octocrab::{Octocrab, OctocrabBuilder};
use serde::{Deserialize, Serialize};
use tracing::error;

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
pub async fn get_client(config: &Config) -> Result<Octocrab> {
    let client = get_client_builder(config).await?.build()?;

    Ok(client)
}

/// Get an authenticated GitHub client builder.
pub async fn get_client_builder(config: &Config) -> Result<OctocrabBuilder> {
    let token = get_authentication_credentials(config).await?;
    Ok(Octocrab::builder().personal_token(token))
}

pub(crate) fn is_client_enabled(config: &Config) -> bool {
    !config.api_disable_client
}

fn now() -> u64 {
    let start = SystemTime::now();
    let duration = start.duration_since(UNIX_EPOCH).expect("time collapsed");

    duration.as_secs()
}

async fn get_authentication_credentials(config: &Config) -> Result<String> {
    if config.github_api_token.is_empty() {
        create_installation_access_token(config).await
    } else {
        Ok(config.github_api_token.clone())
    }
}

fn create_jwt_token(config: &Config) -> Result<String> {
    // GitHub App authentication documentation
    // https://docs.github.com/en/developers/apps/authenticating-with-github-apps#authenticating-as-a-github-app

    let key = EncodingKey::from_rsa_pem(config.github_app_private_key.as_bytes()).unwrap();

    let now_ts = now();
    let claims = JwtClaims {
        // Issued at time
        iat: now_ts,
        // Expiration time, 1 minute
        exp: now_ts + 60,
        // GitHub App Identifier
        iss: config.github_app_id,
    };

    match encode(&Header::new(Algorithm::RS256), &claims, &key) {
        Err(e) => {
            error!("Error while generating JWT: {}", e);
            Err(APIError::JWTCreationError(e.to_string()))
        }
        Ok(s) => Ok(s),
    }
}

async fn create_installation_access_token(config: &Config) -> Result<String> {
    let auth_token = create_jwt_token(config)?;
    let client = Octocrab::builder().personal_token(auth_token).build()?;
    let installation_id = config.github_app_installation_id;

    let resp: InstallationTokenResponse = client
        .post(
            client.absolute_url(&format!(
                "/app/installations/{}/access_tokens",
                installation_id
            ))?,
            None::<&()>,
        )
        .await?;
    Ok(resp.token)
}
