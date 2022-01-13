//! External API validator.

use std::sync::Arc;

use actix_http::http::StatusCode;
use actix_web::{dev::ServiceRequest, web, Error};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use github_scbot_crypto::{CryptoError, JwtUtils};
use github_scbot_database::models::{ExternalAccountModel, ExternalJwtClaims, IDatabaseAdapter};
use github_scbot_sentry::{sentry, WrapEyre};
use thiserror::Error;

use crate::server::AppContext;

/// Validation error.
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Unknown account.")]
    UnknownAccount,
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
    let target_account = extract_account_from_auth(ctx.db_adapter.as_ref(), &credentials).await?;

    // Validate token with ISS
    let tok = credentials.token();
    let _claims: ExternalJwtClaims = JwtUtils::verify_jwt(tok, &target_account.public_key)
        .map_err(|e| ValidationError::token_error(tok, e))?;

    Ok(req)
}

/// Extract account from auth.
pub async fn extract_account_from_auth(
    db_adapter: &dyn IDatabaseAdapter,
    credentials: &BearerAuth,
) -> Result<ExternalAccountModel, ValidationError> {
    extract_account_from_token(db_adapter, credentials.token()).await
}

pub async fn extract_account_from_token(
    db_adapter: &dyn IDatabaseAdapter,
    token: &str,
) -> Result<ExternalAccountModel, ValidationError> {
    let claims: ExternalJwtClaims =
        JwtUtils::decode_jwt(token).map_err(|e| ValidationError::token_error(token, e))?;
    db_adapter
        .external_account()
        .get_from_username(&claims.iss)
        .await
        .map_err(|_e| ValidationError::UnknownAccount)
}
