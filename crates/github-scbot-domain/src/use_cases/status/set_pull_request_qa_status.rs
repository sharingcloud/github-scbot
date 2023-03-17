//! External module.

use github_scbot_config::Config;
use github_scbot_database_interface::DbService;
use github_scbot_domain_models::{ExternalAccount, QaStatus, RepositoryPath};
use github_scbot_ghapi_interface::ApiService;
use github_scbot_lock_interface::LockService;

use crate::{
    commands::{
        commands::{BotCommand, SetQaStatusCommand},
        CommandContext, CommandExecutor,
    },
    Result,
};

/// Set QA status for multiple pull request numbers.
pub struct SetPullRequestQaStatusUseCase<'a> {
    pub config: &'a Config,
    pub api_service: &'a dyn ApiService,
    pub db_service: &'a mut dyn DbService,
    pub lock_service: &'a dyn LockService,
    pub external_account: &'a ExternalAccount,
    pub repository_path: RepositoryPath,
    pub pull_request_numbers: &'a [u64],
    pub author: &'a str,
    pub status: QaStatus,
}

impl<'a> SetPullRequestQaStatusUseCase<'a> {
    #[tracing::instrument(
        skip_all,
        fields(
            repository_path = %self.repository_path,
            pr_numbers = ?self.pull_request_numbers,
            username = %self.author,
            status = ?self.status
        )
    )]
    pub async fn run(&mut self) -> Result<()> {
        let (repo_owner, repo_name) = self.repository_path.components();
        if self
            .db_service
            .external_account_rights_get(repo_owner, repo_name, &self.external_account.username)
            .await?
            .is_some()
        {
            for pr_number in self.pull_request_numbers {
                if self
                    .db_service
                    .pull_requests_get(repo_owner, repo_name, *pr_number)
                    .await?
                    .is_some()
                {
                    let upstream_pr = self
                        .api_service
                        .pulls_get(repo_owner, repo_name, *pr_number)
                        .await?;

                    let mut ctx = CommandContext {
                        config: self.config,
                        api_service: self.api_service,
                        db_service: self.db_service,
                        lock_service: self.lock_service,
                        repo_owner,
                        repo_name,
                        pr_number: *pr_number,
                        upstream_pr: &upstream_pr,
                        comment_id: 0,
                        comment_author: self.author,
                    };

                    let result = SetQaStatusCommand::new(self.status)
                        .handle(&mut ctx)
                        .await?;
                    CommandExecutor::process_command_result(&mut ctx, &result).await?;
                }
            }
        }

        Ok(())
    }
}
