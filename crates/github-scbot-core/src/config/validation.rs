//! Validation utilities.

use std::fmt::Write;

use crate::crypto::JwtUtils;

use super::{config::Config, Result};

enum ApiConfigError {
    MissingToken,
    MissingAppId,
    MissingInstallationId,
    MissingPrivateKey,
    InvalidPrivateKey,
}

use super::errors::EnvVarsSnafu;

fn validate_env_vars(config: &Config) -> Result<()> {
    #[inline]
    fn _missing(error: &mut String, name: &str) {
        error.push('\n');
        write!(error, "  - Missing env. var.: {}", name).unwrap();
    }

    #[inline]
    fn _invalid_key(error: &mut String, name: &str) {
        error.push('\n');
        write!(error, "  - Invalid private key: {}", name).unwrap();
    }

    let mut error = String::new();

    // Check server configuration
    if config.server_bind_ip.is_empty() {
        _missing(&mut error, "BOT_SERVER_BIND_IP");
    }
    if config.server_bind_port == 0 {
        _missing(&mut error, "BOT_SERVER_BIND_PORT");
    }
    if config.bot_username.is_empty() {
        _missing(&mut error, "BOT_USERNAME");
    }
    if config.database_url.is_empty() {
        _missing(&mut error, "DATABASE_URL");
    }

    // Check redis configuration
    if config.redis_address.is_empty() {
        _missing(&mut error, "BOT_REDIS_ADDRESS");
    }

    // Check API credentials: token or private key
    match validate_api_credentials(config) {
        Err(ApiConfigError::MissingToken) => {
            _missing(&mut error, "ENV_GITHUB_API_TOKEN");
        }
        Err(ApiConfigError::MissingAppId) => {
            _missing(&mut error, "ENV_GITHUB_APP_ID");
        }
        Err(ApiConfigError::InvalidPrivateKey) => {
            _invalid_key(&mut error, "ENV_GITHUB_APP_PRIVATE_KEY");
        }
        Err(ApiConfigError::MissingInstallationId) => {
            _missing(&mut error, "ENV_GITHUB_APP_INSTALLATION_ID");
        }
        _ => (),
    }

    if error.is_empty() {
        Ok(())
    } else {
        EnvVarsSnafu { errors: error }.fail()
    }
}

fn validate_api_credentials(config: &Config) -> Result<(), ApiConfigError> {
    // Check token first
    if config.github_api_token.is_empty() {
        match validate_github_app_config(config) {
            // If private key is missing, you might want to use token instead.
            Err(ApiConfigError::MissingPrivateKey) => Err(ApiConfigError::MissingToken),
            res => res,
        }
    } else {
        Ok(())
    }
}

fn validate_github_app_config(config: &Config) -> Result<(), ApiConfigError> {
    // Check Private key
    if config.github_app_private_key.is_empty() {
        Err(ApiConfigError::MissingPrivateKey)
    } else {
        match JwtUtils::parse_encoding_key(&config.github_app_private_key) {
            Err(_) => Err(ApiConfigError::InvalidPrivateKey),
            Ok(_) => {
                // Check App ID
                if config.github_app_id == 0 {
                    Err(ApiConfigError::MissingAppId)
                } else if config.github_app_installation_id == 0 {
                    Err(ApiConfigError::MissingInstallationId)
                } else {
                    Ok(())
                }
            }
        }
    }
}

/// Validate configuration.
pub fn validate_configuration(config: &Config) -> Result<()> {
    validate_env_vars(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    // RSA key specifically generated for these tests.
    const SAMPLE_RSA_KEY: &str = r"
-----BEGIN RSA PUBLIC KEY-----
MIIBigKCAYEAzEWMCHfwGGXxwFDRtHn43opUTW/qMXUoLH7KLpO0meL9jv/TNnI5
totrx/AbnqpKI50TNpYKfw08C9/WC3SZMuyudBOSShXmDjq1yVOM7p9+gjjw5O78
60WqyiUbxOHOIz4CfgoEr23h9I916SCGzqEVTCHvlDE5qQcdNoHeYdohWUTMGxKs
iRMbbHsNvD56zJ8U4AOjOb4J2410ZMx+VQGXeFtZvWYL2EFq1ZiGoo1ZIUZPRImO
axGG0RhzwQdaiktCP7ENjwpr5MBsKlwXFOEb6LdeaCAOqOd05qf4yphzBbLiLK7Y
CZbQ5S3QVQMrn0ycdtFlWt0kAVps9WdB+8izDehuN+pozTm+mjehFsEEj4REGyHu
H3iwEyuGr90vKWEht1Wfvt9C4guBhoLQlSwzgTqNgbHDXiasITmMUwzsgxyASxop
7ih/0aNRO/HfV7rQgFwMrCfPijZJkQHyougprERZJD6U9pPvAIow3G535LpT7mwC
2zEcABBQBwtxAgMBAAE=
-----END RSA PUBLIC KEY-----";

    #[test]
    fn test_validate_github_app_config() {
        let mut config = Config::from_env();

        macro_rules! test {
            ($val_id: tt, $val_iid: tt, $val_pk: tt, $($res: tt)+) => {{
                config.github_app_id = $val_id.into();
                config.github_app_installation_id = $val_iid.into();
                config.github_app_private_key = $val_pk.into();
                assert!(matches!(validate_github_app_config(&config), $($res)+))
            }};
        }

        test!(0_u64, 0_u64, "", Err(ApiConfigError::MissingPrivateKey));
        test!(0_u64, 0_u64, "toto", Err(ApiConfigError::InvalidPrivateKey));
        test!(0_u64, 1_u64, "", Err(ApiConfigError::MissingPrivateKey));
        test!(0_u64, 1_u64, "toto", Err(ApiConfigError::InvalidPrivateKey));
        test!(1234_u64, 0_u64, "", Err(ApiConfigError::MissingPrivateKey));
        test!(
            1234_u64,
            0_u64,
            "toto",
            Err(ApiConfigError::InvalidPrivateKey)
        );
        test!(
            0_u64,
            0_u64,
            SAMPLE_RSA_KEY,
            Err(ApiConfigError::MissingAppId)
        );
        test!(
            1234_u64,
            0_u64,
            SAMPLE_RSA_KEY,
            Err(ApiConfigError::MissingInstallationId)
        );
        test!(1234_u64, 1_u64, SAMPLE_RSA_KEY, Ok(()));
    }

    #[test]
    fn test_validate_api_credentials() {
        let mut config = Config::from_env();

        macro_rules! test {
            ($val_tok: tt, $val_id: tt, $val_iid: tt, $val_pk: tt, $($res: tt)+) => {{
                config.github_api_token = $val_tok.into();
                config.github_app_id = $val_id.into();
                config.github_app_installation_id = $val_iid.into();
                config.github_app_private_key = $val_pk.into();
                assert!(matches!(validate_api_credentials(&config), $($res)+));
            }};
        }

        test!("", 0_u64, 0_u64, "", Err(ApiConfigError::MissingToken));
        test!(
            "",
            0_u64,
            0_u64,
            "iamapkey",
            Err(ApiConfigError::InvalidPrivateKey)
        );
        test!(
            "",
            0_u64,
            0_u64,
            SAMPLE_RSA_KEY,
            Err(ApiConfigError::MissingAppId)
        );
        test!("", 1234_u64, 0_u64, "", Err(ApiConfigError::MissingToken));
        test!(
            "",
            1234_u64,
            0_u64,
            "iamapkey",
            Err(ApiConfigError::InvalidPrivateKey)
        );
        test!(
            "",
            1234_u64,
            0_u64,
            SAMPLE_RSA_KEY,
            Err(ApiConfigError::MissingInstallationId)
        );
        test!("", 1234_u64, 1_u64, SAMPLE_RSA_KEY, Ok(()));
        test!("iamatoken", 0_u64, 0_u64, "", Ok(()));
        test!("iamatoken", 0_u64, 0_u64, "iamapkey", Ok(()));
        test!("iamatoken", 1234_u64, 0_u64, "", Ok(()));
        test!("iamatoken", 1234_u64, 0_u64, "iamapkey", Ok(()));
        test!("iamatoken", 1234_u64, 0_u64, SAMPLE_RSA_KEY, Ok(()));
    }
}
