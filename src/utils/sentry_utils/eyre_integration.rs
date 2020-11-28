//! Sentry eyre integration

use sentry_core::types::Uuid;
use sentry_core::{ClientOptions, Hub, Integration};

/// The Sentry eyre Integration.
#[derive(Debug, Default)]
pub struct EyreIntegration;

impl EyreIntegration {
    /// Creates a new eyre Integration.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Integration for EyreIntegration {
    fn name(&self) -> &'static str {
        "eyre"
    }

    fn setup(&self, cfg: &mut ClientOptions) {
        cfg.in_app_exclude.push("eyre::");
    }
}

/// Captures an `eyre::Report`.
///
/// # Arguments
///
/// * `e` - Report
///
/// # Returns
///
/// * Event UUID
///
#[allow(dead_code)]
pub fn capture_eyre(e: &eyre::Report) -> Uuid {
    Hub::with_active(|hub| hub.capture_eyre(e))
}

/// Hub extension methods for working with `eyre`.
pub trait EyreHubExt {
    /// Captures an `eyre::Report` on a specific hub.
    fn capture_eyre(&self, e: &eyre::Report) -> Uuid;
}

impl EyreHubExt for Hub {
    fn capture_eyre(&self, e: &eyre::Report) -> Uuid {
        let e: &dyn std::error::Error = e.as_ref();
        self.capture_error(e)
    }
}
