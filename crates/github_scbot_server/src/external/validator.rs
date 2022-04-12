//! External API validator.

use std::sync::Arc;

use actix_web::http::StatusCode;
use actix_web::{dev::ServiceRequest, web, Error};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use github_scbot_crypto::{CryptoError, JwtUtils};
use github_scbot_database2::{DbService, ExternalAccount, ExternalAccountDB, ExternalJwtClaims, DatabaseError};
use github_scbot_sentry::{sentry, WrapEyre};
use thiserror::Error;

use crate::server::AppContext;

/// Validation error.
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Unknown account.")]
    UnknownAccount,
    #[error("Database error.")]
    DatabaseError(#[from] DatabaseError),
    #[error(transparent)]
    TokenError(#[from] CryptoError),
}

impl ValidationError {
    pub fn token_error(token: &str, source: github_scbot_crypto::CryptoError) -> Self {
        sentry::configure_scope(|scope| {
            scope.set_extra("Token", token.into());
        });

        Self::TokenError(source)
    }
}

impl From<ValidationError> for WrapEyre {
    fn from(e: ValidationError) -> Self {
        Self::new(e.into(), StatusCode::BAD_REQUEST)
    }
}

/// Jwt authentication validator.
pub async fn jwt_auth_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, Error> {
    jwt_auth_validator_inner(req, credentials)
        .await
        .map_err(WrapEyre::to_http_error)
}

async fn jwt_auth_validator_inner(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, ValidationError> {
    let ctx = req.app_data::<web::Data<Arc<AppContext>>>().unwrap();
    let target_account = extract_account_from_auth(&mut *ctx.db_adapter.external_accounts(), &credentials).await?;

    // Validate token with ISS
    let tok = credentials.token();
    let _claims: ExternalJwtClaims = JwtUtils::verify_jwt(tok, &target_account.public_key())
        .map_err(|e| ValidationError::token_error(tok, e))?;

    Ok(req)
}

/// Extract account from auth.
pub async fn extract_account_from_auth(
    exa_db: &mut dyn ExternalAccountDB,
    credentials: &BearerAuth,
) -> Result<ExternalAccount, ValidationError> {
    extract_account_from_token(exa_db, credentials.token()).await
}

pub async fn extract_account_from_token(
    exa_db: &mut dyn ExternalAccountDB,
    token: &str,
) -> Result<ExternalAccount, ValidationError> {
    let claims: ExternalJwtClaims =
        JwtUtils::decode_jwt(token).map_err(|e| ValidationError::token_error(token, e))?;
    exa_db.get(&claims.iss)
        .await?
        .ok_or_else(|| ValidationError::UnknownAccount)
}
