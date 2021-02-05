//! Validation utilities.

use jsonwebtoken::EncodingKey;

use crate::{
    constants::{
        ENV_BIND_IP, ENV_BIND_PORT, ENV_BOT_USERNAME, ENV_DATABASE_URL, ENV_GITHUB_API_TOKEN,
        ENV_GITHUB_APP_ID, ENV_GITHUB_APP_PRIVATE_KEY,
    },
    CoreError, Result,
};

enum EnvError {
    Missing,
}

enum ApiConfigError {
    MissingToken,
    MissingAppID,
    MissingPrivateKey,
    InvalidPrivateKey,
}

fn check_env_var(name: &str) -> Result<(), EnvError> {
    let entry: String = std::env::var(name).unwrap_or_default();
    if entry.is_empty() {
        Err(EnvError::Missing)
    } else {
        Ok(())
    }
}

fn validate_env_vars() -> Result<()> {
    #[inline]
    fn _missing(error: &mut String, name: &str) {
        error.push('\n');
        error.push_str(&format!("  - Missing env. var.: {}", name));
    }

    #[inline]
    fn _invalid_key(error: &mut String, name: &str) {
        error.push('\n');
        error.push_str(&format!("  - Invalid private key: {}", name));
    }

    // Check mandatory env vars
    let mut error = String::new();
    for name in &[
        ENV_BIND_IP,
        ENV_BIND_PORT,
        ENV_BOT_USERNAME,
        ENV_DATABASE_URL,
    ] {
        if let Err(EnvError::Missing) = check_env_var(name) {
            _missing(&mut error, name);
        }
    }

    // Check API credentials: token or private key
    match validate_api_credentials() {
        Err(ApiConfigError::MissingToken) => {
            _missing(&mut error, ENV_GITHUB_API_TOKEN);
        }
        Err(ApiConfigError::MissingAppID) => {
            _missing(&mut error, ENV_GITHUB_APP_ID);
        }
        Err(ApiConfigError::InvalidPrivateKey) => {
            _invalid_key(&mut error, ENV_GITHUB_APP_PRIVATE_KEY)
        }
        _ => (),
    }

    if error.is_empty() {
        Ok(())
    } else {
        Err(CoreError::ConfigurationError(error))
    }
}

fn validate_api_credentials() -> Result<(), ApiConfigError> {
    // Check token first
    let token: String = std::env::var(crate::constants::ENV_GITHUB_API_TOKEN).unwrap_or_default();
    if token.is_empty() {
        match validate_github_app_config() {
            // If private key is missing, you might want to use token instead.
            Err(ApiConfigError::MissingPrivateKey) => Err(ApiConfigError::MissingToken),
            res => res,
        }
    } else {
        Ok(())
    }
}

fn validate_github_app_config() -> Result<(), ApiConfigError> {
    // Check Private key
    let entry: String =
        std::env::var(crate::constants::ENV_GITHUB_APP_PRIVATE_KEY).unwrap_or_default();
    if entry.is_empty() {
        Err(ApiConfigError::MissingPrivateKey)
    } else {
        match EncodingKey::from_rsa_pem(entry.as_bytes()) {
            Err(_) => Err(ApiConfigError::InvalidPrivateKey),
            Ok(_) => {
                // Check App ID
                if std::env::var(ENV_GITHUB_APP_ID)
                    .unwrap_or_default()
                    .is_empty()
                {
                    Err(ApiConfigError::MissingAppID)
                } else {
                    Ok(())
                }
            }
        }
    }
}

/// Validate configuration.
pub fn validate_configuration() -> Result<()> {
    validate_env_vars()
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
        macro_rules! test {
            ($val_id: tt, $val_pk: tt, $($res: tt)+) => {{
                let env_n1 = ENV_GITHUB_APP_ID;
                let env_n2 = ENV_GITHUB_APP_PRIVATE_KEY;
                ::std::env::set_var(env_n1, $val_id);
                ::std::env::set_var(env_n2, $val_pk);
                assert!(matches!(validate_github_app_config(), $($res)+))
            }};
        }

        test!("", "", Err(ApiConfigError::MissingPrivateKey));
        test!("", "toto", Err(ApiConfigError::InvalidPrivateKey));
        test!("id", "", Err(ApiConfigError::MissingPrivateKey));
        test!("id", "toto", Err(ApiConfigError::InvalidPrivateKey));
        test!("", SAMPLE_RSA_KEY, Err(ApiConfigError::MissingAppID));
        test!("id", SAMPLE_RSA_KEY, Ok(()));
    }

    #[test]
    fn test_validate_api_credentials() {
        macro_rules! test {
            ($val_tok: tt, $val_id: tt, $val_pk: tt, $($res: tt)+) => {{
                let env_n1 = ENV_GITHUB_API_TOKEN;
                let env_n2 = ENV_GITHUB_APP_ID;
                let env_n3 = ENV_GITHUB_APP_PRIVATE_KEY;
                ::std::env::set_var(env_n1, $val_tok);
                ::std::env::set_var(env_n2, $val_id);
                ::std::env::set_var(env_n3, $val_pk);
                assert!(matches!(validate_api_credentials(), $($res)+));
            }};
        }

        test!("", "", "", Err(ApiConfigError::MissingToken));
        test!("", "", "iamapkey", Err(ApiConfigError::InvalidPrivateKey));
        test!("", "", SAMPLE_RSA_KEY, Err(ApiConfigError::MissingAppID));
        test!("", "id", "", Err(ApiConfigError::MissingToken));
        test!("", "id", "iamapkey", Err(ApiConfigError::InvalidPrivateKey));
        test!("", "id", SAMPLE_RSA_KEY, Ok(()));
        test!("iamatoken", "", "", Ok(()));
        test!("iamatoken", "", "iamapkey", Ok(()));
        test!("iamatoken", "id", "", Ok(()));
        test!("iamatoken", "id", "iamapkey", Ok(()));
        test!("iamatoken", "id", SAMPLE_RSA_KEY, Ok(()));
    }

    #[test]
    fn test_check_env_var() {
        macro_rules! test {
            ($env: tt, $val: tt, $($res: tt)+) => {{
                ::std::env::set_var($env, $val);
                assert!(matches!(check_env_var($env), $($res)+));
            }};
        }

        test!(ENV_BIND_IP, "", Err(EnvError::Missing));
        test!(ENV_BIND_IP, "1234", Ok(()));
    }
}
