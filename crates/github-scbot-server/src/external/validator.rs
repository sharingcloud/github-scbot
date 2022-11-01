//! External API validator.

use std::sync::Arc;

use actix_web::http::StatusCode;
use actix_web::ResponseError;
use actix_web::{dev::ServiceRequest, web, Error};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use github_scbot_core::crypto::{CryptoError, JwtUtils};
use github_scbot_core::sentry::sentry;
use github_scbot_database::{DatabaseError, ExternalAccount, ExternalAccountDB, ExternalJwtClaims};
use snafu::{ResultExt, Snafu};

use crate::server::AppContext;

/// Validation error.
#[derive(Debug, Snafu)]
pub enum ValidationError {
    #[snafu(display("Unknown account."))]
    UnknownAccount,
    #[snafu(display("Database error,\n  caused by: {}", source))]
    DatabaseError { source: DatabaseError },
    #[snafu(display("Token error,\n  caused by: {}", source))]
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
        match extract_account_from_auth(&mut *ctx.db_adapter.external_accounts(), &credentials)
            .await
        {
            Ok(acc) => acc,
            Err(e) => return Err((e, req)),
        };

    // Validate token with ISS
    let tok = credentials.token();
    let _claims: ExternalJwtClaims = match JwtUtils::verify_jwt(tok, target_account.public_key()) {
        Ok(claims) => claims,
        Err(e) => return Err((ValidationError::token_error(tok, e), req)),
    };

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
    exa_db
        .get(&claims.iss)
        .await
        .context(DatabaseSnafu)?
        .ok_or_else(|| UnknownAccountSnafu.build())
}
