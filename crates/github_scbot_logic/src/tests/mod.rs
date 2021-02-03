mod logic;
mod reviews;

use github_scbot_api::constants::ENV_API_DISABLE_CLIENT;
use github_scbot_core::constants::ENV_BOT_USERNAME;

fn test_init() {
    dotenv::dotenv().unwrap();

    std::env::set_var(ENV_BOT_USERNAME, "test-bot");
    std::env::set_var(ENV_API_DISABLE_CLIENT, "1");
}
