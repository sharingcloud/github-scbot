//! Test utils

const TEST_BOT_USERNAME: &str = "test-bot";

/// Startup test function.
pub(crate) fn test_init() {
    use crate::webhook::constants::ENV_BOT_USERNAME;

    dotenv::dotenv().ok();
    std::env::set_var(ENV_BOT_USERNAME, TEST_BOT_USERNAME);
}
