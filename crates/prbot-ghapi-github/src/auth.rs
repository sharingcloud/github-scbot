//! Auth.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use http::{header, HeaderMap};
use lazy_static::lazy_static;
use prbot_config::Config;
use prbot_crypto::JwtUtils;
use prbot_ghapi_interface::{ApiService, Result};
use reqwest::ClientBuilder;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::errors::GitHubError;

const INSTALLATION_TOKEN_LIFETIME_IN_SECONDS: u64 = 3600;
const INSTALLATION_TOKEN_RENEW_THRESHOLD: f32 = 0.5;

struct LastInstallationToken {
    token: String,
    expiration: u64,
}

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

lazy_static! {
    static ref LAST_TOKEN: RwLock<LastInstallationToken> = RwLock::new(LastInstallationToken {
        token: String::new(),
        expiration: 0
    });
}

/// Get an authenticated GitHub client builder.
pub async fn get_authenticated_client_builder(
    config: &Config,
    api_service: &dyn ApiService,
) -> Result<ClientBuilder, GitHubError> {
    let builder = get_anonymous_client_builder(config)?;
    let token = get_authentication_credentials(config, api_service).await?;

    let mut headers = HeaderMap::new();
    headers.insert(
        header::ACCEPT,
        header::HeaderValue::from_static("application/vnd.github.squirrel-girl-preview"),
    );
    headers.insert(
        header::AUTHORIZATION,
        header::HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
    );

    Ok(builder.default_headers(headers))
}

/// Get anonymous GitHub client builder.
pub fn get_anonymous_client_builder(config: &Config) -> Result<ClientBuilder, GitHubError> {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::ACCEPT,
        header::HeaderValue::from_static("application/vnd.github.squirrel-girl-preview"),
    );

    Ok(ClientBuilder::new()
        .connect_timeout(Duration::from_millis(config.api.github.connect_timeout))
        .user_agent(format!("prbot/{}", config.version))
        .default_headers(headers))
}

/// Build a GitHub URL.
pub fn build_github_url<T: Into<String>>(config: &Config, path: T) -> String {
    format!("{}{}", config.api.github.root_url, path.into())
}

async fn get_authentication_credentials(
    config: &Config,
    api_service: &dyn ApiService,
) -> Result<String, GitHubError> {
    if config.api.github.token.is_empty() {
        get_or_create_installation_access_token(config, api_service).await
    } else {
        Ok(config.api.github.token.clone())
    }
}

async fn get_or_create_installation_access_token(
    config: &Config,
    api_service: &dyn ApiService,
) -> Result<String, GitHubError> {
    let (last_token, last_expiration) = {
        let auth = LAST_TOKEN.read().await;
        (auth.token.clone(), auth.expiration)
    };

    let now_timestamp = now_timestamp();
    if now_timestamp
        > last_expiration.saturating_sub(
            (INSTALLATION_TOKEN_LIFETIME_IN_SECONDS as f32 * INSTALLATION_TOKEN_RENEW_THRESHOLD)
                as u64,
        )
    {
        // Time to rebuild!
        let token = create_installation_access_token(config, api_service).await?;
        let mut last_auth = LAST_TOKEN.write().await;
        last_auth.token = token.clone();
        last_auth.expiration = now_timestamp + INSTALLATION_TOKEN_LIFETIME_IN_SECONDS;

        Ok(token)
    } else {
        Ok(last_token)
    }
}

fn now_timestamp() -> u64 {
    let start = SystemTime::now();
    let duration = start.duration_since(UNIX_EPOCH).expect("time collapsed");
    duration.as_secs()
}

fn create_app_token(config: &Config) -> Result<String, GitHubError> {
    // GitHub App authentication documentation
    // https://docs.github.com/en/developers/apps/authenticating-with-github-apps#authenticating-as-a-github-app

    let now_ts = now_timestamp();
    let claims = JwtClaims {
        // Issued at time
        iat: now_ts,
        // Expiration time, 1 minute
        exp: now_ts + 60,
        // GitHub App Identifier
        iss: config.api.github.app_id,
    };

    JwtUtils::create_jwt(&config.api.github.app_private_key, &claims)
        .map_err(|e| GitHubError::ImplementationError { source: e.into() })
}

#[tracing::instrument(skip_all)]
async fn create_installation_access_token(
    config: &Config,
    api_service: &dyn ApiService,
) -> Result<String, GitHubError> {
    let auth_token = create_app_token(config)?;
    api_service
        .installations_create_token(&auth_token, config.api.github.app_installation_id)
        .await
        .map_err(|e| GitHubError::ImplementationError { source: e.into() })
}

#[cfg(test)]
mod tests {
    use prbot_crypto::RsaUtils;
    use prbot_ghapi_interface::MockApiService;

    use super::*;

    fn arrange_config() -> Config {
        let mut config = Config::from_env_no_version();
        config.api.github.app_id = 1234;
        config.api.github.token = "abcdef".into();
        config.api.github.app_installation_id = 1234;

        let (pri_key, _) = RsaUtils::generate_rsa_keys();
        config.api.github.app_private_key = pri_key.to_string();
        config
    }

    #[test]
    fn test_create_app_token() {
        let config = arrange_config();
        let token = create_app_token(&config).unwrap();
        let decoded_token: JwtClaims = JwtUtils::decode_jwt(&token).unwrap();

        assert_eq!(decoded_token.exp - decoded_token.iat, 60);
        assert_eq!(decoded_token.iss, 1234);
    }

    #[tokio::test]
    async fn test_create_installation_access_token() {
        let config = arrange_config();

        let mut adapter = MockApiService::new();
        adapter
            .expect_installations_create_token()
            .once()
            .withf(|auth_token, installation_id| !auth_token.is_empty() && installation_id == &1234)
            .returning(|_, _| Ok("this-is-a-token".into()));

        assert_eq!(
            create_installation_access_token(&config, &adapter)
                .await
                .unwrap(),
            "this-is-a-token"
        );
    }

    #[tokio::test]
    async fn test_get_authentication_credentials() {
        let mut config = arrange_config();
        let adapter = MockApiService::new();

        // Should use api token
        assert_eq!(
            get_authentication_credentials(&config, &adapter)
                .await
                .unwrap(),
            "abcdef"
        );

        config.api.github.token = "".into();

        let mut adapter = MockApiService::new();
        adapter
            .expect_installations_create_token()
            .once()
            .withf(|auth_token, installation_id| !auth_token.is_empty() && installation_id == &1234)
            .returning(|_, _| Ok("token".into()));

        // Should create installation access token
        assert_eq!(
            get_authentication_credentials(&config, &adapter)
                .await
                .unwrap(),
            "token"
        );
    }

    #[tokio::test]
    async fn test_get_authenticated_client_builder() {
        let config = arrange_config();
        let api_service = MockApiService::new();

        get_authenticated_client_builder(&config, &api_service)
            .await
            .unwrap()
            .build()
            .unwrap();
    }
}
