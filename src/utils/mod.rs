//! Common utils

mod sentry_utils;

#[cfg(test)]
mod test;

pub use sentry_utils::with_sentry_configuration;

#[cfg(test)]
pub(crate) use test::test_init;
