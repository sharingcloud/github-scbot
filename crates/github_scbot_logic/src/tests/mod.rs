use github_scbot_core::Config;

mod commands;
mod reviews;

fn test_config() -> Config {
    let mut config = Config::from_env();
    config.bot_username = "test-bot".into();
    config.api_disable_client = true;
    config
}
