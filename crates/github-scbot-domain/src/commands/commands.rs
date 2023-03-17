use async_trait::async_trait;
use github_scbot_core::config::Config;
use github_scbot_database_interface::DbService;
use github_scbot_ghapi_interface::{types::GhPullRequest, ApiService};
use github_scbot_lock_interface::LockService;

use crate::{commands::command::CommandExecutionResult, Result};

mod admin;
mod user;
pub use admin::*;
pub use user::*;

pub struct CommandContext<'a> {
    pub config: &'a Config,
    pub api_service: &'a dyn ApiService,
    pub db_service: &'a mut dyn DbService,
    pub lock_service: &'a dyn LockService,
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
    use github_scbot_database_memory::MemoryDb;
    use github_scbot_ghapi_interface::MockApiService;
    use github_scbot_lock_interface::MockLockService;

    use super::*;

    pub(crate) struct CommandContextTest {
        pub config: Config,
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
                config: Config::from_env(),
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

        pub fn as_context(&mut self) -> CommandContext {
            CommandContext {
                config: &self.config,
                api_service: &self.api_service,
                db_service: &mut self.db_service,
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
