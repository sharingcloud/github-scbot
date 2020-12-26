//! Test utils

/// Startup test function.
pub(crate) fn test_init() {
    dotenv::dotenv().ok();
}
