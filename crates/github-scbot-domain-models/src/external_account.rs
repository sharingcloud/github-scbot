use github_scbot_core::crypto::{CryptoError, JwtUtils, RsaUtils};
use github_scbot_core::utils::TimeUtils;
use serde::{Deserialize, Serialize};

/// External Jwt claims.
#[derive(Debug, Serialize, Deserialize)]
pub struct ExternalJwtClaims {
    /// Issued at time
    pub iat: u64,
    /// Identifier
    pub iss: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExternalAccount {
    pub username: String,
    pub public_key: String,
    pub private_key: String,
}

impl ExternalAccount {
    pub fn generate_access_token(&self) -> Result<String, CryptoError> {
        let now_ts = TimeUtils::now_timestamp();
        let claims = ExternalJwtClaims {
            // Issued at time
            iat: now_ts,
            // Username
            iss: self.username.clone(),
        };

        JwtUtils::create_jwt(&self.private_key, &claims)
    }

    pub fn with_generated_keys(mut self) -> Self {
        let (private_key, public_key) = RsaUtils::generate_rsa_keys();
        self.private_key = private_key.to_string();
        self.public_key = public_key.to_string();
        self
    }
}
