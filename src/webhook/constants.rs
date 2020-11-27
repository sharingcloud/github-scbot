//! Webhook constants

pub const GITHUB_EVENT_HEADER: &str = "X-GitHub-Event";
pub const GITHUB_SIGNATURE_HEADER: &str = "X-Hub-Signature-256";
pub const SIGNATURE_PREFIX_LENGTH: usize = "sha256=".len();

pub const ENV_GITHUB_SECRET: &str = "BOT_GITHUB_SECRET";
pub const ENV_DISABLE_SIGNATURE: &str = "BOT_DISABLE_SIGNATURE_VERIFICATION";
pub const ENV_DISABLE_WELCOME_COMMENTS: &str = "BOT_DISABLE_WELCOME_COMMENTS";
pub const ENV_BOT_USERNAME: &str = "BOT_USERNAME";
