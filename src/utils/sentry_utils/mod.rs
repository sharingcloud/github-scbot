//! Sentry module

use tracing::info;

mod constants;

use crate::errors::Result;

pub fn with_sentry_configuration<T>(func: T) -> Result<()>
where
    T: FnOnce() -> Result<()>,
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
