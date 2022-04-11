//! Eyre report wrapper.

use std::fmt;

use actix_web::{
    http::{header, StatusCode},
    Error, HttpResponse, HttpResponseBuilder, ResponseError,
};
use stable_eyre::eyre;

/// Eyre Report wrapper.
pub struct WrapEyre {
    report: eyre::Report,
    status_code: StatusCode,
}

impl WrapEyre {
    /// Create eyre wrapper.
    pub fn new(report: eyre::Report, status_code: StatusCode) -> Self {
        Self {
            report,
            status_code,
        }
    }

    /// Convert any error
    pub fn to_http_error<E: Into<WrapEyre>>(e: E) -> Error {
        e.into().into()
    }
}

impl fmt::Display for WrapEyre {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.report, f)
    }
}

impl fmt::Debug for WrapEyre {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.report, f)
    }
}

impl std::ops::Deref for WrapEyre {
    type Target = eyre::Report;

    fn deref(&self) -> &Self::Target {
        &self.report
    }
}

impl ResponseError for WrapEyre {
    fn status_code(&self) -> StatusCode {
        self.status_code
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .set_header(header::CONTENT_TYPE, "application/json; charset=utf-8")
            .body(
                serde_json::json!({
                    "error": self.report.to_string()
                })
                .to_string(),
            )
    }
}
