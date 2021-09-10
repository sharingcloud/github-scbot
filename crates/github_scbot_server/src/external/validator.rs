//! External API validator.

use github_scbot_crypto::{decode_jwt, verify_jwt, CryptoError};
use github_scbot_database::models::{
    DatabaseAdapter, ExternalAccountModel, ExternalJwtClaims, IDatabaseAdapter,
};
use github_scbot_libs::{
    actix_web::{dev::ServiceRequest, web, Error},
    actix_web_httpauth::extractors::bearer::BearerAuth,
    sentry,
};
use sentry_actix::eyre::WrapEyre;
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

/// Jwt authentication validator.
pub async fn jwt_auth_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, Error> {
    jwt_auth_validator_inner(req, credentials)
        .await
        .map_err(|e| WrapEyre::bad_request(e).into())
}

async fn jwt_auth_validator_inner(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, ValidationError> {
    let ctx = req.app_data::<web::Data<AppContext>>().unwrap();
    let db_adapter = DatabaseAdapter::new(&ctx.pool);
    let target_account = extract_account_from_auth(&db_adapter, &credentials).await?;

    // Validate token with ISS
    let tok = credentials.token();
    let _claims: ExternalJwtClaims = verify_jwt(tok, &target_account.public_key)
        .map_err(|e| ValidationError::token_error(tok, e))?;

    Ok(req)
}

/// Extract account from auth.
pub async fn extract_account_from_auth(
    db_adapter: &dyn IDatabaseAdapter,
    credentials: &BearerAuth,
) -> Result<ExternalAccountModel, ValidationError> {
    let tok = credentials.token();
    let claims: ExternalJwtClaims =
        decode_jwt(tok).map_err(|e| ValidationError::token_error(tok, e))?;
    db_adapter
        .external_account()
        .get_from_username(&claims.iss)
        .await
        .map_err(|_e| ValidationError::UnknownAccount)
}
