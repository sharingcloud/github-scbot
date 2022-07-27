//! Webhook errors.

use actix_http::StatusCode;
use actix_web::ResponseError;
use github_scbot_core::types::events::EventType;
use snafu::{prelude::*, Backtrace};

/// Webhook error.
#[allow(missing_docs)]
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum ServerError {
    #[snafu(display(
        "Error while parsing webhook event for type {},\n  caused by: {}",
        event_type,
        source
    ))]
    EventParseError {
        event_type: EventType,
        source: serde_json::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Missing webhook signature."))]
    MissingWebhookSignature { backtrace: Backtrace },

    #[snafu(display("Invalid webhook signature."))]
    InvalidWebhookSignature { backtrace: Backtrace },

    #[snafu(display("I/O error,\n  caused by: {}", source))]
    IoError {
        source: std::io::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Logic error,\n  caused by: {}", source))]
    LogicError {
        source: github_scbot_logic::LogicError,
        backtrace: Backtrace,
    },

    #[snafu(display("Internal error."))]
    InternalError,
}

impl ResponseError for ServerError {
    fn status_code(&self) -> StatusCode {
        match &self {
            ServerError::InvalidWebhookSignature { .. } => StatusCode::FORBIDDEN,
            ServerError::MissingWebhookSignature { .. } => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// Result alias for `ServerError`.
pub type Result<T> = core::result::Result<T, ServerError>;