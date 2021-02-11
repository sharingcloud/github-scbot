//! Webhook errors.

use actix_web::{
    dev::HttpResponseBuilder,
    http::{header, StatusCode},
    HttpResponse,
};
use github_scbot_types::events::EventType;
use thiserror::Error;

/// Webhook error.
#[derive(Debug, Error)]
pub enum ServerError {
    /// Event parsing error.
    #[error("Error while parsing webhook event {0:?}: {1}")]
    EventParseError(EventType, serde_json::Error),

    /// Wraps [`std::io::Error`].
    #[error(transparent)]
    IOError(#[from] std::io::Error),

    /// Wraps [`regex::Error`].
    #[error(transparent)]
    RegexError(#[from] regex::Error),

    /// Wraps [`github_scbot_database::DatabaseError`].
    #[error(transparent)]
    DatabaseError(#[from] github_scbot_database::DatabaseError),

    /// Wraps [`github_scbot_logic::LogicError`].
    #[error(transparent)]
    LogicError(#[from] github_scbot_logic::LogicError),
}

impl actix_web::ResponseError for ServerError {
    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .set_header(header::CONTENT_TYPE, "application/json; charset=utf-8")
            .body(serde_json::json!({
                "error": self.to_string()
            }))
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

/// Result alias for `ServerError`.
pub type Result<T> = core::result::Result<T, ServerError>;
