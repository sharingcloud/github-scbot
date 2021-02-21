//! Sentry module.

use stable_eyre::eyre::Report;
use tracing::info;

use crate::Config;

/// Configure Sentry integration by wrapping a function.
///
/// # Arguments
///
/// * `config` - Bot configuration
/// * `func` - Function to wrap.
pub fn with_sentry_configuration<T, E>(config: &Config, func: T) -> Result<(), E>
where
    T: FnOnce() -> Result<(), E>,
{
    if !config.sentry_url.is_empty() {
        info!("Sentry integration enabled.");

        // Enable backtraces
        std::env::set_var("RUST_BACKTRACE", "1");

        // Ignore eyre modules
        let mut options = ::sentry::ClientOptions::new();
        options.in_app_exclude.push("actix_cors");
        options.in_app_exclude.push("actix_http");
        options.in_app_exclude.push("actix_rt");
        options.in_app_exclude.push("actix_server");
        options.in_app_exclude.push("actix_service");
        options.in_app_exclude.push("actix_web");
        options.in_app_exclude.push("actix_web_httpauth");
        options.in_app_exclude.push("eyre");
        options.in_app_exclude.push("futures_util");
        options.in_app_exclude.push("sentry_actix");
        options.in_app_exclude.push("stable_eyre");
        options.in_app_exclude.push("tokio");
        options.release = ::sentry::release_name!();
        options.send_default_pii = true;

        let _guard = ::sentry::init((config.sentry_url.clone(), options));
        func()
    } else {
        func()
    }
}

/// Capture eyre report.
///
/// # Arguments
///
/// * `err` - `eyre` report
pub fn capture_eyre(err: &Report) {
    sentry_eyre::capture_eyre(err);
}
