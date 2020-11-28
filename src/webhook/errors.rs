//! Errors

use std::fmt;

#[derive(Debug)]
pub struct WebhookError {
    err: eyre::Report,
}

impl fmt::Display for WebhookError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        eyre_report_to_sentry(&self.err, f)
    }
}

impl actix_web::error::ResponseError for WebhookError {}

impl From<eyre::Report> for WebhookError {
    fn from(err: eyre::Report) -> Self {
        Self { err }
    }
}

fn eyre_report_to_sentry(err: &eyre::Report, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{:#}", err)
}
