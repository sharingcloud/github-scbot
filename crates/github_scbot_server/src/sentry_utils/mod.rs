//! Sentry module.

use tracing::info;

pub mod constants;

/// Configure Sentry integration by wrapping a function.
///
/// # Arguments
///
/// * `func` - Function to wrap.
pub fn with_sentry_configuration<T, E>(func: T) -> Result<(), E>
where
    T: FnOnce() -> Result<(), E>,
{
    if let Ok(url) = std::env::var(constants::ENV_SENTRY_URL) {
        info!("Sentry integration enabled.");

        // Create client options
        let mut options = sentry::ClientOptions::new();
        options.attach_stacktrace = true;

        let _guard = sentry::init((url, options));
        func()
    } else {
        func()
    }
}
