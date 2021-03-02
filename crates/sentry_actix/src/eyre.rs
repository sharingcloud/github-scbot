//! Eyre report wrapper.

use std::fmt;

use actix_web::{
    dev::HttpResponseBuilder,
    http::{header, StatusCode},
    HttpResponse, ResponseError,
};
use stable_eyre::eyre;

/// Eyre Report wrapper.
pub struct WrapEyre {
    report: eyre::Report,
    status_code: StatusCode,
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

impl<E> From<E> for WrapEyre
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn from(error: E) -> Self {
        Self {
            report: error.into(),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

macro_rules! impl_err {
    ($name: ident, $code: expr) => {
        /// Create a specific error. See [`with_status`] for reference.
        pub fn $name<E: std::error::Error + Send + Sync + 'static>(err: E) -> Self {
            Self::with_status(err, $code)
        }
    };
}

impl WrapEyre {
    impl_err!(bad_request, StatusCode::BAD_REQUEST);

    impl_err!(internal_server_error, StatusCode::INTERNAL_SERVER_ERROR);

    impl_err!(unauthorized, StatusCode::UNAUTHORIZED);

    impl_err!(forbidden, StatusCode::FORBIDDEN);

    /// Create error with status.
    ///
    /// # Arguments
    ///
    /// * `err` - Error
    /// * `status_code` - Status code
    pub fn with_status<E: std::error::Error + Send + Sync + 'static>(
        e: E,
        status_code: StatusCode,
    ) -> Self {
        Self {
            report: e.into(),
            status_code,
        }
    }
}

impl ResponseError for WrapEyre {
    fn status_code(&self) -> StatusCode {
        self.status_code
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .set_header(header::CONTENT_TYPE, "application/json; charset=utf-8")
            .body(serde_json::json!({
                "error": self.report.to_string()
            }))
    }
}
