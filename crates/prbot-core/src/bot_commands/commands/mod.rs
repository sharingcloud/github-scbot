use async_trait::async_trait;

use crate::{bot_commands::command::CommandExecutionResult, Result};

mod admin;
mod context;
mod user;
pub use admin::*;
pub use context::CommandContext;
pub use user::*;

#[async_trait]
pub trait BotCommand {
    async fn handle(&self, ctx: &CommandContext) -> Result<CommandExecutionResult>;
}

#[cfg(test)]
pub(crate) mod tests {
    use prbot_config::Config;
    use prbot_database_memory::MemoryDb;
    use prbot_ghapi_interface::{types::GhPullRequest, MockApiService};
    use prbot_lock_interface::MockLockService;

    use super::*;
    use crate::CoreModule;

    pub(crate) struct CommandContextTest {
        pub config: Config,
        pub core_module: CoreModule,
        pub api_service: MockApiService,
        pub db_service: MemoryDb,
        pub lock_service: MockLockService,
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
                config: Config::from_env_no_version(),
                core_module: CoreModule::builder().build(),
                api_service: MockApiService::new(),
                db_service: MemoryDb::new(),
                lock_service: MockLockService::new(),
                repo_owner: "owner".into(),
                repo_name: "name".into(),
                pr_number: 1,
                upstream_pr: GhPullRequest::default(),
                comment_id: 1,
                comment_author: "me".into(),
            }
        }

        pub fn as_context(&self) -> CommandContext {
            CommandContext {
                config: &self.config,
                core_module: &self.core_module,
                api_service: &self.api_service,
                db_service: &self.db_service,
                lock_service: &self.lock_service,
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
