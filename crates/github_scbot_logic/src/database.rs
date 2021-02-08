//! Database module.

use github_scbot_api::{labels::set_step_label, pulls::get_pull_request};
use github_scbot_core::Config;
use github_scbot_database::{
    models::{PullRequestCreation, PullRequestModel, RepositoryCreation, RepositoryModel},
    DbConn,
};
use github_scbot_types::{common::GHRepository, pulls::GHPullRequest};

use crate::errors::Result;

/// Process GitHub repository.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `repository` - GitHub repository
pub fn process_repository(conn: &DbConn, repository: &GHRepository) -> Result<RepositoryModel> {
    RepositoryModel::get_or_create(
        conn,
        RepositoryCreation {
            name: repository.name.clone(),
            owner: repository.owner.login.clone(),
            ..Default::default()
        },
    )
    .map_err(Into::into)
}

/// Process GitHub pull request.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `repository` - GitHub repository
/// * `pull_request` - GitHub pull request
pub fn process_pull_request(
    conn: &DbConn,
    repository: &GHRepository,
    pull_request: &GHPullRequest,
) -> Result<(RepositoryModel, PullRequestModel)> {
    let repo = process_repository(conn, repository)?;
    let mut upstream = PullRequestCreation::from_upstream(repo.id, &pull_request);
    upstream.needed_reviewers_count = repo.default_needed_reviewers_count;

    let pr = PullRequestModel::get_or_create(conn, upstream)?;

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
    if let Some(pr_model) =
        PullRequestModel::get_from_repository_id_and_number(conn, repo_model.id, pr_number as i32)
    {
        Ok(pr_model)
    } else {
        let pr = get_pull_request(config, &repo_model.owner, &repo_model.name, pr_number).await?;

        let pr_model = PullRequestModel::get_or_create(
            conn,
            PullRequestCreation::from_upstream(repo_model.id, &pr),
        )?;

        Ok(pr_model)
    }
}
