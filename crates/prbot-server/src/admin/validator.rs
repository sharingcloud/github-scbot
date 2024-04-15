//! External API validator.

use std::time::{SystemTime, UNIX_EPOCH};

use actix_web::{dev::ServiceRequest, http::StatusCode, web, Error, ResponseError};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use prbot_config::Config;
use prbot_crypto::{CryptoError, JwtUtils, PrivateRsaKey};
use prbot_sentry::sentry;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::server::AppContext;

/// External Jwt claims.
#[derive(Debug, Serialize, Deserialize)]
pub struct AdminJwtClaims {
    /// Issued at time
    pub iat: u64,
    /// Expiration
    pub exp: u64,
}

/// Validation error.
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Admin disabled")]
    AdminDisabled,
    #[error("Token error,\n  caused by: {}", source)]
    TokenError { source: CryptoError },
    #[error("Token expired")]
    TokenExpired
}

impl ValidationError {
    pub fn token_error(token: &str, source: CryptoError) -> Self {
        sentry::configure_scope(|scope| {
            scope.set_extra("Token", token.into());
        });

        Self::TokenError { source }
    }
}

impl ResponseError for ValidationError {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

/// Admin jwt authentication validator.
pub async fn admin_jwt_auth_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    admin_jwt_auth_validator_inner(req, credentials)
        .await
        .map_err(|(err, req)| (err.into(), req))
}

async fn admin_jwt_auth_validator_inner(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (ValidationError, ServiceRequest)> {
    let ctx = req.app_data::<web::Data<AppContext>>().unwrap();
    if ctx.config.server.admin_private_key.is_empty() {
        return Err((ValidationError::AdminDisabled, req));
    }

    // Validate token
    let pubkey =
        PrivateRsaKey::new(ctx.config.server.admin_private_key.clone()).extract_public_key();
    let tok = credentials.token();
    let claims: AdminJwtClaims = match JwtUtils::verify_jwt(tok, pubkey.as_str()) {
        Ok(claims) => claims,
        Err(e) => return Err((ValidationError::token_error(tok, e), req)),
    };

    if now_timestamp() >= claims.exp {
        return Err((ValidationError::TokenExpired, req));
    }

    Ok(req)
}

fn now_timestamp() -> u64 {
    let start = SystemTime::now();
    let duration = start.duration_since(UNIX_EPOCH).expect("time collapsed");

    duration.as_secs()
}

/// Generate admin token.
pub fn generate_admin_token(config: &Config) -> Result<String, CryptoError> {
    let now_ts = now_timestamp();
    let claims = AdminJwtClaims {
        // Issued at time
        iat: now_ts,
        // Expiration in 24h
        exp: now_ts + (60 * 60 * 24),
    };

    JwtUtils::create_jwt(&config.server.admin_private_key, &claims)
}
