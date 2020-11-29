//! Common utils

mod sentry_utils;

pub use sentry_utils::{capture_eyre, with_sentry_configuration};
