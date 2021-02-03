mod logic;
mod reviews;

use github_scbot_core::constants::{ENV_API_DISABLE_CLIENT, ENV_BOT_USERNAME};

fn test_init() {
    std::env::set_var(ENV_BOT_USERNAME, "test-bot");
    std::env::set_var(ENV_API_DISABLE_CLIENT, "1");
}
