//! External module.

use github_scbot_api::adapter::IAPIAdapter;
use github_scbot_database::models::{ExternalAccountModel, IDatabaseAdapter, RepositoryModel};
use github_scbot_redis::IRedisAdapter;

use crate::{
    commands::{handle_qa_command, CommandExecutor},
    Result,
};

/// Set QA status for multiple pull request numbers.
#[allow(clippy::too_many_arguments)]
pub async fn set_qa_status_for_pull_requests(
    api_adapter: &impl IAPIAdapter,
    db_adapter: &dyn IDatabaseAdapter,
    redis_adapter: &dyn IRedisAdapter,
    account: &ExternalAccountModel,
    repository_path: &str,
    pull_request_numbers: &[u64],
    author: &str,
    status: Option<bool>,
) -> Result<()> {
    let repo = RepositoryModel::get_from_path(db_adapter.repository(), repository_path).await?;
    db_adapter
        .external_account_right()
        .get_right(&account.username, &repo)
        .await?;

    for pr_num in pull_request_numbers {
        let mut pr = db_adapter
            .pull_request()
            .get_from_repository_and_number(&repo, *pr_num)
            .await?;
        let result = handle_qa_command(db_adapter, &mut pr, author, status).await?;
        CommandExecutor::process_command_result(
            api_adapter,
            db_adapter,
            redis_adapter,
            &repo,
            &mut pr,
            0,
            &result,
        )
        .await?;
    }

    Ok(())
}
