//! External module.

use github_scbot_core::Config;
use github_scbot_database::{
    models::{PullRequestModel, RepositoryModel},
    DbConn,
};

use crate::{commands::handle_qa_command, Result};

/// Set QA status for multiple pull request numbers.
///
/// # Arguments
///
/// * `config` - Bot configuration
/// * `conn` - Database connection
/// * `repository_path` - Repository path
/// * `pull_request_numbers` - Pull request numbers
/// * `author` - Action author
/// * `status` - Pass (`Some(True)`) / Fail (`Some(False)`) / Waiting (`None`)
pub async fn set_qa_status_for_pull_requests(
    config: &Config,
    conn: &DbConn,
    repository_path: &str,
    pull_request_numbers: &[u64],
    author: &str,
    status: Option<bool>,
) -> Result<()> {
    if let Some(repo) = RepositoryModel::get_from_path(conn, repository_path)? {
        for pr_num in pull_request_numbers {
            if let Some(mut pr) =
                PullRequestModel::get_from_repository_id_and_number(conn, repo.id, *pr_num as i32)
            {
                handle_qa_command(config, conn, &repo, &mut pr, author, status).await?;
            } else {
                eprintln!(
                    "PR #{} not found for repository {}.",
                    pr_num, repository_path
                );
            }
        }
    } else {
        eprintln!("Repository {} not found.", repository_path);
    }

    Ok(())
}
