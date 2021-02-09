//! External API validator.

use actix_http::{http::StatusCode, ResponseError};
use actix_web::{dev::ServiceRequest, web, Error};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use github_scbot_crypto::{decode_jwt, verify_jwt};
use github_scbot_database::models::{ExternalAccountModel, ExternalJwtClaims};
use thiserror::Error;

use crate::server::AppContext;

/// Validation error.
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Unknown account.")]
    UnknownAccount,
    #[error("JWT decoding failure.")]
    JWTDecodeFailure,
    #[error("JWT verification failure.")]
    JWTVerificationFailure,
    #[error("Database connection failure.")]
    DatabaseConnectionFailure,
}

impl ResponseError for ValidationError {
    fn status_code(&self) -> StatusCode {
        StatusCode::FORBIDDEN
    }
}

/// JWT authentication validator.
pub async fn jwt_auth_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, Error> {
    jwt_auth_validator_inner(req, credentials).map_err(Into::into)
}

fn jwt_auth_validator_inner(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, ValidationError> {
    let ctx = req.app_data::<web::Data<AppContext>>().unwrap();
    let conn = ctx
        .pool
        .get()
        .map_err(|_e| ValidationError::DatabaseConnectionFailure)?;
    let tok = credentials.token();

    let claims: ExternalJwtClaims =
        decode_jwt(tok).map_err(|_e| ValidationError::JWTDecodeFailure)?;
    let target_account = ExternalAccountModel::get_from_username(&conn, &claims.iss)
        .ok_or(ValidationError::UnknownAccount)?;
    let _: ExternalJwtClaims = verify_jwt(tok, &target_account.public_key)
        .map_err(|_e| ValidationError::JWTVerificationFailure)?;

    Ok(req)
}
