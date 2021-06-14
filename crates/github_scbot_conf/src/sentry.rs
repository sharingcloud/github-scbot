//! Sentry module.

use std::future::Future;

use stable_eyre::eyre::Report;
use tracing::info;

use crate::Config;

/// Configure Sentry integration by wrapping a function.
pub async fn with_sentry_configuration<T, Fut, E>(config: &Config, func: T) -> Result<(), E>
where
    T: FnOnce() -> Fut,
    Fut: Future<Output = Result<(), E>>,
{
    let _guard = {
        if config.sentry_url.is_empty() {
            None
        } else {
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

            Some(::sentry::init((config.sentry_url.clone(), options)))
        }
    };

    func().await
}

/// Capture eyre report.
pub fn capture_eyre(err: &Report) {
    sentry_eyre::capture_eyre(err);
}
