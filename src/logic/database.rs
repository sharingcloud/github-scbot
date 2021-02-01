//! Database module.

use crate::{
    api::labels::set_step_label,
    database::{
        models::{PullRequestCreation, PullRequestModel, RepositoryCreation, RepositoryModel},
        DbConn,
    },
    logic::errors::Result,
    types::{common::GHRepository, pull_requests::GHPullRequest},
};

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
    let pr = PullRequestModel::get_or_create(
        conn,
        PullRequestCreation {
            repository_id: repo.id,
            name: pull_request.title.clone(),
            number: pull_request.number as i32,
            ..Default::default()
        },
    )?;

    Ok((repo, pr))
}

/// Apply pull request step.
///
/// # Arguments
///
/// * `repository_model` - Repository model
/// * `pr_model` - Pull request model
pub async fn apply_pull_request_step(
    repository_model: &RepositoryModel,
    pr_model: &PullRequestModel,
) -> Result<()> {
    set_step_label(
        &repository_model.owner,
        &repository_model.name,
        pr_model.get_number(),
        pr_model.get_step_label(),
    )
    .await
    .map_err(Into::into)
}
