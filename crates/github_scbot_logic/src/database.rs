//! Database module.

use github_scbot_api::{labels::set_step_label, pulls::get_pull_request};
use github_scbot_conf::Config;
use github_scbot_database::{
    models::{PullRequestModel, RepositoryModel},
    DbConn,
};
use github_scbot_types::{common::GHRepository, pulls::GHPullRequest};

use crate::errors::Result;

/// Process GitHub repository.
///
/// # Arguments
///
/// * `config` - Application config
/// * `conn` - Database connection
/// * `repository` - GitHub repository
pub fn process_repository(
    config: &Config,
    conn: &DbConn,
    repository: &GHRepository,
) -> Result<RepositoryModel> {
    RepositoryModel::builder_from_github(config, repository)
        .create_or_update(conn)
        .map_err(Into::into)
}

/// Process GitHub pull request.
///
/// # Arguments
///
/// * `config` - Application config
/// * `conn` - Database connection
/// * `repository` - GitHub repository
/// * `pull_request` - GitHub pull request
pub fn process_pull_request(
    config: &Config,
    conn: &DbConn,
    repository: &GHRepository,
    pull_request: &GHPullRequest,
) -> Result<(RepositoryModel, PullRequestModel)> {
    let repo = process_repository(config, conn, repository)?;
    let pr = match PullRequestModel::builder_from_github(&repo, pull_request).create_or_update(conn)
    {
        Ok(pr) => pr,
        Err(_) => {
            // Handle duplicate keys
            PullRequestModel::get_from_repository_and_number(conn, &repo, pull_request.number)?
        }
    };

    Ok((repo, pr))
}

/// Apply pull request step.
///
/// # Arguments
///
/// * `config` - Bot configuration
/// * `repository_model` - Repository model
/// * `pr_model` - Pull request model
pub async fn apply_pull_request_step(
    config: &Config,
    repository_model: &RepositoryModel,
    pr_model: &PullRequestModel,
) -> Result<()> {
    set_step_label(
        config,
        &repository_model.owner,
        &repository_model.name,
        pr_model.get_number(),
        pr_model.get_step_label(),
    )
    .await
    .map_err(Into::into)
}

/// Get or fetch pull request from ID.
///
/// # Arguments
///
/// * `config` - Bot configuration
/// * `conn` - Database connection
/// * `repo_model` - Repository model
/// * `pr_number` - Pull request number
pub async fn get_or_fetch_pull_request(
    config: &Config,
    conn: &DbConn,
    repo_model: &RepositoryModel,
    pr_number: u64,
) -> Result<PullRequestModel> {
    // Try fetching pull request
    if let Ok(pr_model) =
        PullRequestModel::get_from_repository_and_number(conn, repo_model, pr_number)
    {
        Ok(pr_model)
    } else {
        let pr = get_pull_request(config, &repo_model.owner, &repo_model.name, pr_number).await?;
        let pr_model =
            PullRequestModel::builder_from_github(&repo_model, &pr).create_or_update(conn)?;

        Ok(pr_model)
    }
}
