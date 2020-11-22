//! Webhook logic

use eyre::Result;

use super::types::{PullRequest, Repository};
use crate::database::models::{
    DbConn, PullRequestCreation, PullRequestModel, RepositoryCreation, RepositoryModel,
};

pub fn process_repository(conn: &DbConn, repo: &Repository) -> Result<()> {
    RepositoryModel::get_or_create(
        conn,
        &RepositoryCreation {
            name: &repo.name,
            owner: &repo.owner.login,
        },
    )?;

    Ok(())
}

pub fn process_pull_request(conn: &DbConn, repo: &Repository, pull: &PullRequest) -> Result<()> {
    let repo = RepositoryModel::get_or_create(
        conn,
        &RepositoryCreation {
            name: &repo.name,
            owner: &repo.owner.login,
        },
    )?;

    PullRequestModel::get_or_create(
        conn,
        &PullRequestCreation {
            repository_id: repo.id,
            name: &pull.title,
            number: pull.number,
            automerge: false,
            step: "none",
        },
    )?;

    Ok(())
}
