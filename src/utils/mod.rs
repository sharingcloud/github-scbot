//! Common utils

mod sentry_utils;

#[cfg(test)]
mod tests;

pub use sentry_utils::with_sentry_configuration;
#[cfg(test)]
pub(crate) use tests::test_init;
