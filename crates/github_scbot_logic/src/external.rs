//! External module.

use github_scbot_conf::Config;
use github_scbot_database::{
    get_connection,
    models::{ExternalAccountModel, ExternalAccountRightModel, PullRequestModel, RepositoryModel},
    DbPool,
};

use crate::{commands::handle_qa_command, Result};

/// Set QA status for multiple pull request numbers.
pub async fn set_qa_status_for_pull_requests(
    config: &Config,
    pool: DbPool,
    account: &ExternalAccountModel,
    repository_path: &str,
    pull_request_numbers: &[u64],
    author: &str,
    status: Option<bool>,
) -> Result<()> {
    let repo = RepositoryModel::get_from_path(pool.clone(), repository_path.to_owned()).await?;

    let conn = get_connection(&pool)?;
    ExternalAccountRightModel::get_right(&conn, &account.username, &repo)?;

    for pr_num in pull_request_numbers {
        let mut pr = PullRequestModel::get_from_repository_and_number(&conn, &repo, *pr_num)?;

        handle_qa_command(config, &conn, &repo, &mut pr, author, status).await?;
    }

    Ok(())
}
