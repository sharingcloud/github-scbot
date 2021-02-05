//! Utilities.

use std::time::{SystemTime, UNIX_EPOCH};

use github_scbot_core::constants::{
    ENV_API_DISABLE_CLIENT, ENV_GITHUB_API_TOKEN, ENV_GITHUB_APP_ID,
    ENV_GITHUB_APP_INSTALLATION_ID, ENV_GITHUB_APP_PRIVATE_KEY,
};
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
pub async fn get_client() -> Result<Octocrab> {
    let client = get_client_builder().await?.build()?;

    Ok(client)
}

/// Get an authenticated GitHub client builder.
pub async fn get_client_builder() -> Result<OctocrabBuilder> {
    let token = get_authentication_credentials().await?;
    Ok(Octocrab::builder().personal_token(token))
}

pub(crate) fn is_client_enabled() -> bool {
    std::env::var(ENV_API_DISABLE_CLIENT).ok().is_none()
}

fn now() -> u64 {
    let start = SystemTime::now();
    let duration = start.duration_since(UNIX_EPOCH).expect("time collapsed");

    duration.as_secs()
}

async fn get_authentication_credentials() -> Result<String> {
    let token: String = std::env::var(ENV_GITHUB_API_TOKEN).unwrap_or_default();
    if token.is_empty() {
        create_installation_access_token().await
    } else {
        Ok(token)
    }
}

fn create_jwt_token() -> Result<String> {
    // GitHub App authentication documentation
    // https://docs.github.com/en/developers/apps/authenticating-with-github-apps#authenticating-as-a-github-app

    let key = EncodingKey::from_rsa_pem(
        std::env::var(ENV_GITHUB_APP_PRIVATE_KEY)
            .unwrap()
            .as_bytes(),
    )
    .unwrap();

    let now_ts = now();
    let claims = JwtClaims {
        // Issued at time
        iat: now_ts,
        // Expiration time, 10 minutes maximum, enforced by GitHub
        exp: now_ts + (60 * 10),
        // GitHub App Identifier
        iss: std::env::var(ENV_GITHUB_APP_ID).unwrap().parse().unwrap(),
    };

    match encode(&Header::new(Algorithm::RS256), &claims, &key) {
        Err(e) => {
            error!("Error while generating JWT: {}", e);
            Err(APIError::JWTCreationError(e.to_string()))
        }
        Ok(s) => Ok(s),
    }
}

async fn create_installation_access_token() -> Result<String> {
    let auth_token = create_jwt_token()?;
    let client = Octocrab::builder().personal_token(auth_token).build()?;
    let installation_id = std::env::var(ENV_GITHUB_APP_INSTALLATION_ID).unwrap_or_default();

    let resp = client
        ._post(
            client.absolute_url(&format!(
                "/app/installations/{}/access_tokens",
                installation_id
            ))?,
            None::<&()>,
        )
        .await?;
    let content = resp.json::<InstallationTokenResponse>().await.unwrap();
    Ok(content.token)
}
