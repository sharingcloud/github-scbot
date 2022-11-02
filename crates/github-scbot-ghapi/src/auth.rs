//! Auth.

use std::time::Duration;

use github_scbot_core::{config::Config, crypto::JwtUtils, utils::TimeUtils};
use http::{header, HeaderMap};
use reqwest::ClientBuilder;
use serde::{Deserialize, Serialize};

use crate::{adapter::ApiService, Result, ApiError};

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

/// Get an authenticated GitHub client builder.
pub async fn get_authenticated_client_builder(
    config: &Config,
    api_adapter: &dyn ApiService,
) -> Result<ClientBuilder> {
    let builder = get_anonymous_client_builder(config)?;
    let token = get_authentication_credentials(config, api_adapter).await?;

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
pub fn get_anonymous_client_builder(config: &Config) -> Result<ClientBuilder> {
    const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

    let mut headers = HeaderMap::new();
    headers.insert(
        header::ACCEPT,
        header::HeaderValue::from_static("application/vnd.github.squirrel-girl-preview"),
    );

    Ok(ClientBuilder::new()
        .connect_timeout(Duration::from_millis(config.github_api_connect_timeout))
        .user_agent(format!("github-scbot/{APP_VERSION}"))
        .default_headers(headers))
}

/// Build a GitHub URL.
pub fn build_github_url<T: Into<String>>(config: &Config, path: T) -> String {
    format!("{}{}", config.github_api_root_url, path.into())
}

async fn get_authentication_credentials(
    config: &Config,
    api_adapter: &dyn ApiService,
) -> Result<String> {
    if config.github_api_token.is_empty() {
        create_installation_access_token(config, api_adapter).await
    } else {
        Ok(config.github_api_token.clone())
    }
}

fn create_app_token(config: &Config) -> Result<String> {
    // GitHub App authentication documentation
    // https://docs.github.com/en/developers/apps/authenticating-with-github-apps#authenticating-as-a-github-app

    let now_ts = TimeUtils::now_timestamp();
    let claims = JwtClaims {
        // Issued at time
        iat: now_ts,
        // Expiration time, 1 minute
        exp: now_ts + 60,
        // GitHub App Identifier
        iss: config.github_app_id,
    };

    JwtUtils::create_jwt(&config.github_app_private_key, &claims).map_err(|e| ApiError::JwtError { source: e })
}

async fn create_installation_access_token(
    config: &Config,
    api_adapter: &dyn ApiService,
) -> Result<String> {
    let auth_token = create_app_token(config)?;
    api_adapter
        .installations_create_token(&auth_token, config.github_app_installation_id)
        .await
}

#[cfg(test)]
mod tests {
    use github_scbot_core::crypto::{JwtUtils, RsaUtils};

    use crate::adapter::MockApiService;

    use super::*;

    fn arrange_config() -> Config {
        let mut config = Config::from_env();
        config.github_app_id = 1234;
        config.github_api_token = "123456".into();

        let (pri_key, _) = RsaUtils::generate_rsa_keys();
        config.github_app_private_key = pri_key.to_string();
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

    #[actix_rt::test]
    async fn test_create_installation_access_token() {
        let config = arrange_config();

        let mut adapter = MockApiService::new();
        adapter
            .expect_installations_create_token()
            .times(1)
            .returning(|_, _| Ok("this-is-a-token".into()));

        assert_eq!(
            create_installation_access_token(&config, &adapter)
                .await
                .unwrap(),
            "this-is-a-token"
        );
    }

    #[actix_rt::test]
    async fn test_get_authentication_credentials() {
        let mut config = arrange_config();

        let mut adapter = MockApiService::new();
        adapter
            .expect_installations_create_token()
            .times(0)
            .returning(|_, _| Ok("token".into()));

        // Should use api token
        assert_eq!(
            get_authentication_credentials(&config, &adapter)
                .await
                .unwrap(),
            "123456"
        );

        config.github_api_token = "".into();

        let mut adapter = MockApiService::new();
        adapter
            .expect_installations_create_token()
            .times(1)
            .returning(|_, _| Ok("token".into()));

        // Should create installation access token
        assert_eq!(
            get_authentication_credentials(&config, &adapter)
                .await
                .unwrap(),
            "token"
        );
    }

    #[actix_rt::test]
    async fn test_get_authenticated_client_builder() {
        let config = arrange_config();

        let mut adapter = MockApiService::new();
        adapter
            .expect_installations_create_token()
            .times(0)
            .returning(|_, _| Ok("token".into()));

        get_authenticated_client_builder(&config, &adapter)
            .await
            .unwrap()
            .build()
            .unwrap();
    }
}
