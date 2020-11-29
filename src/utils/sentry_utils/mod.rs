//! Sentry module

use eyre::Result;
use log::info;

mod constants;
mod eyre_integration;

pub use eyre_integration::capture_eyre;

pub fn with_sentry_configuration<T>(func: T) -> Result<()>
where
    T: FnOnce() -> Result<()>,
{
    if let Ok(url) = std::env::var(constants::ENV_SENTRY_URL) {
        info!("Sentry integration enabled.");

        // Create client options
        let mut options =
            sentry::ClientOptions::new().add_integration(eyre_integration::EyreIntegration::new());
        options.attach_stacktrace = true;

        let _guard = sentry::init((url, options));
        func()
    } else {
        func()
    }
}
