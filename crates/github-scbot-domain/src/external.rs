//! External module.

use github_scbot_core::{
    config::Config,
    types::{repository::RepositoryPath, status::QaStatus},
};
use github_scbot_database::{DbServiceAll, ExternalAccount};
use github_scbot_ghapi::adapter::ApiService;
use github_scbot_redis::RedisService;

use crate::{
    commands::{
        commands::{BotCommand, SetQaStatusCommand},
        CommandContext, CommandExecutor,
    },
    Result,
};

/// Set QA status for multiple pull request numbers.
#[allow(clippy::too_many_arguments)]
#[tracing::instrument(
    skip_all,
    fields(
        repository_path = %repository_path,
        pr_numbers = ?pull_request_numbers,
        username = %author,
        status = ?status
    )
)]
pub async fn set_qa_status_for_pull_requests(
    config: &Config,
    api_adapter: &dyn ApiService,
    db_adapter: &mut dyn DbServiceAll,
    redis_adapter: &dyn RedisService,
    account: &ExternalAccount,
    repository_path: RepositoryPath,
    pull_request_numbers: &[u64],
    author: &str,
    status: QaStatus,
) -> Result<()> {
    let (repo_owner, repo_name) = repository_path.components();
    if db_adapter
        .external_account_rights_get(repo_owner, repo_name, &account.username)
        .await?
        .is_some()
    {
        for pr_number in pull_request_numbers {
            if db_adapter
                .pull_requests_get(repo_owner, repo_name, *pr_number)
                .await?
                .is_some()
            {
                let upstream_pr = api_adapter
                    .pulls_get(repo_owner, repo_name, *pr_number)
                    .await?;

                let mut ctx = CommandContext {
                    config,
                    api_adapter,
                    db_adapter,
                    redis_adapter,
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
