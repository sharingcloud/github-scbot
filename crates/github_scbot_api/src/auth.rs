//! Auth.

use github_scbot_conf::Config;
use github_scbot_crypto::{create_jwt, now};
use octocrab::{Octocrab, OctocrabBuilder};
use serde::{Deserialize, Serialize};

use crate::{adapter::IAPIAdapter, ApiError, Result};

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
pub async fn get_client_builder(
    config: &Config,
    api_adapter: &impl IAPIAdapter,
) -> Result<OctocrabBuilder> {
    let token = get_authentication_credentials(config, api_adapter).await?;
    Ok(Octocrab::builder().personal_token(token))
}

/// Get uninitialized client.
pub fn get_uninitialized_client() -> Result<Octocrab> {
    Octocrab::builder().build().map_err(ApiError::from)
}

async fn get_authentication_credentials(
    config: &Config,
    api_adapter: &impl IAPIAdapter,
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

    let now_ts = now();
    let claims = JwtClaims {
        // Issued at time
        iat: now_ts,
        // Expiration time, 1 minute
        exp: now_ts + 60,
        // GitHub App Identifier
        iss: config.github_app_id,
    };

    create_jwt(&config.github_app_private_key, &claims)
        .map_err(|e| ApiError::JWTError(e.to_string()))
}

async fn create_installation_access_token(
    config: &Config,
    api_adapter: &impl IAPIAdapter,
) -> Result<String> {
    let auth_token = create_app_token(config)?;
    api_adapter
        .installations_create_token(&auth_token, config.github_app_installation_id)
        .await
}

#[cfg(test)]
mod tests {
    use github_scbot_crypto::{decode_jwt, generate_rsa_keys};

    use super::*;
    use crate::adapter::DummyAPIAdapter;

    fn arrange_config() -> Config {
        let mut config = Config::from_env();
        config.github_app_id = 1234;
        config.github_api_token = "123456".into();

        let (pri_key, _) = generate_rsa_keys();
        config.github_app_private_key = pri_key;
        config
    }

    #[test]
    fn test_get_uninitialized_client() {
        get_uninitialized_client().unwrap();
    }

    #[test]
    fn test_create_app_token() {
        let config = arrange_config();
        let token = create_app_token(&config).unwrap();
        let decoded_token: JwtClaims = decode_jwt(&token).unwrap();

        assert_eq!(decoded_token.exp - decoded_token.iat, 60);
        assert_eq!(decoded_token.iss, 1234);
    }

    #[actix_rt::test]
    async fn test_create_installation_access_token() {
        let config = arrange_config();
        let mut adapter = DummyAPIAdapter::new();
        adapter
            .installations_create_token_response
            .set_response(Ok("this-is-a-token".to_string()));

        assert_eq!(
            create_installation_access_token(&config, &adapter)
                .await
                .unwrap(),
            "this-is-a-token"
        );
        assert!(adapter.installations_create_token_response.called());
    }

    #[actix_rt::test]
    async fn test_get_authentication_credentials() {
        let mut config = arrange_config();
        let mut adapter = DummyAPIAdapter::new();
        adapter
            .installations_create_token_response
            .set_response(Ok("token".to_string()));

        // Should use api token
        assert_eq!(
            get_authentication_credentials(&config, &adapter)
                .await
                .unwrap(),
            "123456"
        );
        assert!(!adapter.installations_create_token_response.called());

        config.github_api_token = "".into();

        // Should create installation access token
        assert_eq!(
            get_authentication_credentials(&config, &adapter)
                .await
                .unwrap(),
            "token"
        );
        assert!(adapter.installations_create_token_response.called());
    }

    #[actix_rt::test]
    async fn test_get_client_builder() {
        let config = arrange_config();
        let adapter = DummyAPIAdapter::new();
        get_client_builder(&config, &adapter)
            .await
            .unwrap()
            .build()
            .unwrap();

        assert!(!adapter.installations_create_token_response.called());
    }
}
