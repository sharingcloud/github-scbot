use prbot_config::Config;
use prbot_database_interface::DbService;
use prbot_ghapi_interface::{types::GhPullRequest, ApiService};
use prbot_lock_interface::LockService;
use prbot_models::{PullRequestHandle, RepositoryPath};

use crate::{CoreContext, CoreModule};

pub struct CommandContext<'a> {
    pub config: &'a Config,
    pub core_module: &'a CoreModule,
    pub api_service: &'a (dyn ApiService + 'a),
    pub db_service: &'a (dyn DbService + 'a),
    pub lock_service: &'a (dyn LockService + 'a),
    pub repo_owner: &'a str,
    pub repo_name: &'a str,
    pub pr_number: u64,
    pub upstream_pr: &'a GhPullRequest,
    pub comment_id: u64,
    pub comment_author: &'a str,
}

impl<'a> CommandContext<'a> {
    pub fn repository_path(&self) -> RepositoryPath {
        (self.repo_owner, self.repo_name).into()
    }

    pub fn pr_handle(&self) -> PullRequestHandle {
        (self.repo_owner, self.repo_name, self.pr_number).into()
    }

    pub fn as_core_context(&self) -> CoreContext<'a> {
        CoreContext {
            config: self.config,
            core_module: self.core_module,
            api_service: self.api_service,
            db_service: self.db_service,
            lock_service: self.lock_service,
        }
    }
}
