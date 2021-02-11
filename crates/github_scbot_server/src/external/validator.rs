//! External API validator.

use actix_web::{
    dev::{HttpResponseBuilder, ServiceRequest},
    http::{header, StatusCode},
    web, Error, HttpResponse, ResponseError,
};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use github_scbot_crypto::{decode_jwt, verify_jwt};
use github_scbot_database::{
    models::{ExternalAccountModel, ExternalJwtClaims},
    DbConn,
};
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
    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .set_header(header::CONTENT_TYPE, "application/json; charset=utf-8")
            .body(serde_json::json!({
                "error": self.to_string()
            }))
    }

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
    let target_account = extract_account_from_auth(&conn, &credentials)?;

    // Validate token with ISS
    let tok = credentials.token();
    let _: ExternalJwtClaims = verify_jwt(tok, &target_account.public_key)
        .map_err(|_e| ValidationError::JWTVerificationFailure)?;

    Ok(req)
}

/// Extract account from auth.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `credentials` - Auth credentials
pub fn extract_account_from_auth(
    conn: &DbConn,
    credentials: &BearerAuth,
) -> Result<ExternalAccountModel, ValidationError> {
    let tok = credentials.token();
    let claims: ExternalJwtClaims =
        decode_jwt(tok).map_err(|_e| ValidationError::JWTDecodeFailure)?;
    ExternalAccountModel::get_from_username(&conn, &claims.iss)
        .map_err(|_e| ValidationError::UnknownAccount)
}
