//! External module.

use github_scbot_database2::{DbService, ExternalAccount};
use github_scbot_ghapi::adapter::ApiService;
use github_scbot_redis::RedisService;
use github_scbot_types::repository::RepositoryPath;

use crate::{
    commands::{handle_qa_command, CommandExecutor},
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
    api_adapter: &dyn ApiService,
    db_adapter: &dyn DbService,
    redis_adapter: &dyn RedisService,
    account: &ExternalAccount,
    repository_path: RepositoryPath,
    pull_request_numbers: &[u64],
    author: &str,
    status: Option<bool>,
) -> Result<()> {
    let (repo_owner, repo_name) = repository_path.components();
    if db_adapter
        .external_account_rights()
        .get(repo_owner, repo_name, account.username())
        .await?
        .is_some()
    {
        for pr_number in pull_request_numbers {
            if db_adapter
                .pull_requests()
                .get(repo_owner, repo_name, *pr_number)
                .await?
                .is_some()
            {
                let result = handle_qa_command(
                    db_adapter, repo_owner, repo_name, *pr_number, author, status,
                )
                .await?;

                CommandExecutor::process_command_result(
                    api_adapter,
                    db_adapter,
                    redis_adapter,
                    repo_owner,
                    repo_name,
                    *pr_number,
                    0,
                    &result,
                )
                .await?;
            }
        }
    }

    Ok(())
}
