//! Common utils

mod sentry_utils;
mod startup;

pub use sentry_utils::{capture_eyre, with_sentry_configuration};
pub use startup::configure_startup;
