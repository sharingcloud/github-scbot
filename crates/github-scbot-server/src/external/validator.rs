//! External API validator.

use std::sync::Arc;

use actix_web::{dev::ServiceRequest, http::StatusCode, web, Error, ResponseError};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use github_scbot_crypto::{CryptoError, JwtUtils};
use github_scbot_database_interface::{DatabaseError, DbService};
use github_scbot_domain_models::{ExternalAccount, ExternalJwtClaims};
use github_scbot_sentry::sentry;
use thiserror::Error;

use crate::server::AppContext;

/// Validation error.
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Unknown account.")]
    UnknownAccount,
    #[error("Database error,\n  caused by: {}", source)]
    DatabaseError { source: DatabaseError },
    #[error("Token error,\n  caused by: {}", source)]
    TokenError { source: CryptoError },
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

/// Jwt authentication validator.
pub async fn jwt_auth_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    jwt_auth_validator_inner(req, credentials)
        .await
        .map_err(|(err, req)| (err.into(), req))
}

async fn jwt_auth_validator_inner(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (ValidationError, ServiceRequest)> {
    let ctx = req.app_data::<web::Data<Arc<AppContext>>>().unwrap();
    let target_account =
        extract_account_from_auth(ctx.db_service.lock().await.as_mut(), &credentials).await;
    let target_account = match target_account {
        Ok(acc) => acc,
        Err(e) => return Err((e, req)),
    };

    // Validate token with ISS
    let tok = credentials.token();
    let _claims: ExternalJwtClaims = match JwtUtils::verify_jwt(tok, &target_account.public_key) {
        Ok(claims) => claims,
        Err(e) => return Err((ValidationError::token_error(tok, e), req)),
    };

    Ok(req)
}

/// Extract account from auth.
pub async fn extract_account_from_auth(
    db_service: &mut dyn DbService,
    credentials: &BearerAuth,
) -> Result<ExternalAccount, ValidationError> {
    extract_account_from_token(db_service, credentials.token()).await
}

pub async fn extract_account_from_token(
    db_service: &mut dyn DbService,
    token: &str,
) -> Result<ExternalAccount, ValidationError> {
    let claims: ExternalJwtClaims =
        JwtUtils::decode_jwt(token).map_err(|e| ValidationError::token_error(token, e))?;
    db_service
        .external_accounts_get(&claims.iss)
        .await
        .map_err(|e| ValidationError::DatabaseError { source: e })?
        .ok_or(ValidationError::UnknownAccount)
}
