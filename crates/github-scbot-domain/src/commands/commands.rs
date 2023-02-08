use github_scbot_core::config::Config;
use github_scbot_core::types::pulls::GhPullRequest;
use github_scbot_database::DbService;
use github_scbot_ghapi::adapter::ApiService;
use github_scbot_redis::RedisService;

use async_trait::async_trait;

use crate::commands::command::CommandExecutionResult;
use crate::Result;

mod admin;
mod user;
pub use admin::*;
pub use user::*;

pub struct CommandContext<'a> {
    pub config: &'a Config,
    pub api_adapter: &'a dyn ApiService,
    pub db_adapter: &'a mut dyn DbService,
    pub redis_adapter: &'a dyn RedisService,
    pub repo_owner: &'a str,
    pub repo_name: &'a str,
    pub pr_number: u64,
    pub upstream_pr: &'a GhPullRequest,
    pub comment_id: u64,
    pub comment_author: &'a str,
}

#[async_trait(?Send)]
pub trait BotCommand {
    async fn handle(&self, ctx: &mut CommandContext) -> Result<CommandExecutionResult>;
}

#[cfg(test)]
pub(crate) mod tests {
    use github_scbot_core::{config::Config, types::pulls::GhPullRequest};
    use github_scbot_database::MemoryDb;
    use github_scbot_ghapi::adapter::MockApiService;
    use github_scbot_redis::MockRedisService;

    use super::CommandContext;

    pub(crate) struct CommandContextTest {
        pub config: Config,
        pub api_adapter: MockApiService,
        pub db_adapter: MemoryDb,
        pub redis_adapter: MockRedisService,
        pub repo_owner: String,
        pub repo_name: String,
        pub pr_number: u64,
        pub upstream_pr: GhPullRequest,
        pub comment_id: u64,
        pub comment_author: String,
    }

    impl CommandContextTest {
        pub fn new() -> Self {
            Self {
                config: Config::from_env(),
                api_adapter: MockApiService::new(),
                db_adapter: MemoryDb::new(),
                redis_adapter: MockRedisService::new(),
                repo_owner: "owner".into(),
                repo_name: "name".into(),
                pr_number: 1,
                upstream_pr: GhPullRequest::default(),
                comment_id: 1,
                comment_author: "me".into(),
            }
        }

        pub fn as_context(&mut self) -> CommandContext {
            CommandContext {
                config: &self.config,
                api_adapter: &self.api_adapter,
                db_adapter: &mut self.db_adapter,
                redis_adapter: &self.redis_adapter,
                repo_owner: &self.repo_owner,
                repo_name: &self.repo_name,
                pr_number: self.pr_number,
                upstream_pr: &self.upstream_pr,
                comment_id: self.comment_id,
                comment_author: &self.comment_author,
            }
        }
    }
}
