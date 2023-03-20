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
    pub db_service: &'a dyn DbService,
    pub lock_service: &'a dyn LockService,
}

impl<'a> SetPullRequestQaStatusUseCase<'a> {
    #[tracing::instrument(
        skip_all,
        fields(
            external_account = external_account.username,
            repository_path = %repository_path,
            pr_numbers = ?pull_request_numbers,
            author = %author,
            status = ?status
        )
    )]
    pub async fn run(
        &self,
        external_account: &ExternalAccount,
        repository_path: RepositoryPath,
        pull_request_numbers: &[u64],
        author: &str,
        status: QaStatus,
    ) -> Result<()> {
        let (repo_owner, repo_name) = repository_path.components();
        if self
            .db_service
            .external_account_rights_get(repo_owner, repo_name, &external_account.username)
            .await?
            .is_some()
        {
            for pr_number in pull_request_numbers {
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
                        comment_author: author,
                    };

                    let result = SetQaStatusCommand::new(status).handle(&mut ctx).await?;
                    CommandExecutor::process_command_result(&mut ctx, &result).await?;
                }
            }
        }

        Ok(())
    }
}
