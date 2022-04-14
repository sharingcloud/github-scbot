//! External module.

use github_scbot_database2::{DbService, ExternalAccount};
use github_scbot_ghapi::adapter::IAPIAdapter;
use github_scbot_redis::IRedisAdapter;
use github_scbot_types::repository::RepositoryPath;

use crate::{
    commands::{handle_qa_command, CommandExecutor},
    Result,
};

/// Set QA status for multiple pull request numbers.
#[allow(clippy::too_many_arguments)]
#[tracing::instrument(skip(api_adapter, db_adapter, redis_adapter, account))]
pub async fn set_qa_status_for_pull_requests(
    api_adapter: &dyn IAPIAdapter,
    db_adapter: &dyn DbService,
    redis_adapter: &dyn IRedisAdapter,
    account: &ExternalAccount,
    repository_path: RepositoryPath,
    pull_request_numbers: &[u64],
    author: &str,
    status: Option<bool>,
) -> Result<()> {
    let (owner, name) = repository_path.components();
    if let Some(_) = db_adapter
        .external_account_rights()
        .get(owner, name, account.username())
        .await?
    {
        for pr_num in pull_request_numbers {
            if let Some(pr) = db_adapter.pull_requests().get(owner, name, *pr_num).await? {
                let repo = db_adapter.repositories().get(owner, name).await?.unwrap();
                let result =
                    handle_qa_command(db_adapter, owner, name, *pr_num, author, status).await?;
                let upstream_pr = api_adapter.pulls_get(owner, name, *pr_num).await?;

                CommandExecutor::process_command_result(
                    api_adapter,
                    db_adapter,
                    redis_adapter,
                    &repo,
                    &pr,
                    &upstream_pr,
                    0,
                    &result,
                )
                .await?;
            }
        }
    }

    Ok(())
}
