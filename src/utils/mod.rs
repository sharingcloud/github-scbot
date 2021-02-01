//! Common utils.

pub mod sentry_utils;

#[cfg(test)]
mod tests;
#[cfg(test)]
pub(crate) use tests::test_init;
