//! Database

use crate::api::labels::set_step_label;
use crate::database::models::{
    DbConn, PullRequestCreation, PullRequestModel, RepositoryCreation, RepositoryModel,
};
use crate::webhook::errors::Result;
use crate::webhook::types::{PullRequest, Repository};

pub fn process_repository(conn: &DbConn, repo: &Repository) -> Result<RepositoryModel> {
    RepositoryModel::get_or_create(
        conn,
        &RepositoryCreation {
            name: &repo.name,
            owner: &repo.owner.login,
        },
    )
    .map_err(Into::into)
}

#[allow(clippy::clippy::cast_possible_truncation)]
pub fn process_pull_request(
    conn: &DbConn,
    repo: &Repository,
    pull: &PullRequest,
) -> Result<(RepositoryModel, PullRequestModel)> {
    let repo = process_repository(conn, repo)?;
    let pr = PullRequestModel::get_or_create(
        conn,
        &PullRequestCreation {
            repository_id: repo.id,
            name: &pull.title,
            number: pull.number as i32,
            automerge: false,
            check_status: None,
            step: None,
        },
    )?;

    Ok((repo, pr))
}

#[allow(clippy::cast_sign_loss)]
pub async fn apply_pull_request_step(
    repo_model: &RepositoryModel,
    pr_model: &PullRequestModel,
) -> Result<()> {
    set_step_label(
        &repo_model.owner,
        &repo_model.name,
        pr_model.number as u64,
        pr_model.step_enum(),
    )
    .await
    .map_err(Into::into)
}
