//! External API validator.

use actix_web::{dev::ServiceRequest, http::StatusCode, web, Error, ResponseError};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use prbot_crypto::{CryptoError, JwtUtils};
use prbot_database_interface::{DatabaseError, DbService};
use prbot_models::{ExternalAccount, ExternalJwtClaims};
use prbot_sentry::sentry;
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
    let ctx = req.app_data::<web::Data<AppContext>>().unwrap();
    let target_account = extract_account_from_auth(ctx.db_service.as_ref(), &credentials).await;
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
    db_service: &dyn DbService,
    credentials: &BearerAuth,
) -> Result<ExternalAccount, ValidationError> {
    extract_account_from_token(db_service, credentials.token()).await
}

pub async fn extract_account_from_token(
    db_service: &dyn DbService,
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

#[cfg(test)]
mod tests {
    use prbot_crypto::JwtUtils;
    use prbot_database_interface::DbService;
    use prbot_database_memory::MemoryDb;
    use prbot_models::{ExternalAccount, ExternalJwtClaims};

    use super::*;

    #[tokio::test]
    async fn extract() {
        let db_service = MemoryDb::new();
        let external_account = db_service
            .external_accounts_create(
                ExternalAccount {
                    username: "Test".into(),
                    ..Default::default()
                }
                .with_generated_keys(),
            )
            .await
            .unwrap();

        let claims = ExternalJwtClaims {
            iat: 1,
            iss: "Test".into(),
        };

        let token = JwtUtils::create_jwt(&external_account.private_key, &claims).unwrap();
        let extracted_account = extract_account_from_token(&db_service, &token)
            .await
            .unwrap();

        assert_eq!(external_account, extracted_account);
    }
}
