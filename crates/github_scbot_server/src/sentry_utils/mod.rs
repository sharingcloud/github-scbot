//! Sentry module.

use github_scbot_core::Config;
use tracing::info;

pub mod constants;

/// Configure Sentry integration by wrapping a function.
///
/// # Arguments
///
/// * `func` - Function to wrap.
pub fn with_sentry_configuration<T, E>(config: Config, func: T) -> Result<(), E>
where
    T: FnOnce() -> Result<(), E>,
{
    if !config.sentry_url.is_empty() {
        info!("Sentry integration enabled.");

        // Create client options
        let mut options = sentry::ClientOptions::new();
        options.attach_stacktrace = true;

        let _guard = sentry::init((config.sentry_url, options));
        func()
    } else {
        func()
    }
}
