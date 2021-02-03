//! Database import/export module.

use std::{
    collections::HashMap,
    io::{Read, Write},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{
    errors::Result,
    models::{
        PullRequestCreation, PullRequestModel, RepositoryCreation, RepositoryModel, ReviewCreation,
        ReviewModel,
    },
    DbConn,
};

/// Import error.
#[derive(Debug, Error)]
pub enum ImportError {
    /// Wraps [`serde_json::Error`].
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),

    /// Wraps [`std::io::Error`] with a path.
    #[error("IO error on file {0:?}: {1}")]
    IOError(PathBuf, std::io::Error),

    /// Unknown repository ID error.
    #[error("Unknown repository ID in file: {0}")]
    UnknownRepositoryIdError(i32),

    /// Unknown pull request ID error.
    #[error("Unknown pull request ID in file: {0}")]
    UnknownPullRequestIdError(i32),
}

/// Export error.
#[derive(Debug, Error)]
pub enum ExportError {
    /// Wraps [`serde_json::Error`].
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),

    /// Wraps [`std::io::Error`] with a path.
    #[error("IO error on file {0:?}: {1}")]
    IOError(PathBuf, std::io::Error),
}

#[derive(Debug, Deserialize, Serialize)]
struct ImportExportModel {
    repositories: Vec<RepositoryModel>,
    pull_requests: Vec<PullRequestModel>,
    reviews: Vec<ReviewModel>,
}

/// Export database models to JSON.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `writer` - Output stream
pub fn export_models_to_json<W>(conn: &DbConn, writer: &mut W) -> Result<()>
where
    W: Write,
{
    let model = ImportExportModel {
        repositories: RepositoryModel::list(conn)?,
        pull_requests: PullRequestModel::list(conn)?,
        reviews: ReviewModel::list(conn)?,
    };

    serde_json::to_writer_pretty(writer, &model).map_err(ExportError::SerdeError)?;

    Ok(())
}

/// Import database models from JSON.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `reader` - Input stream
pub fn import_models_from_json<R>(conn: &DbConn, reader: R) -> Result<()>
where
    R: Read,
{
    let mut model: ImportExportModel =
        serde_json::from_reader(reader).map_err(ImportError::SerdeError)?;

    let mut repo_id_map = HashMap::new();
    let mut pr_id_map = HashMap::new();

    // Create or update repositories
    for repository in &mut model.repositories {
        println!(
            "> Importing repository {}/{}",
            repository.owner, repository.name
        );

        let repo = RepositoryModel::get_or_create(
            conn,
            RepositoryCreation {
                owner: repository.owner.clone(),
                name: repository.name.clone(),
                ..Default::default()
            },
        )?;
        repo_id_map.insert(repository.id, repo.id);
        repository.id = repo.id;
        repository.save(conn)?;
    }

    // Create or update pull requests
    for pull_request in &mut model.pull_requests {
        println!("> Importing pull request #{}", pull_request.get_number());

        let repo_id = repo_id_map.get(&pull_request.repository_id).ok_or(
            ImportError::UnknownRepositoryIdError(pull_request.repository_id),
        )?;
        let pr = PullRequestModel::get_or_create(
            conn,
            PullRequestCreation {
                repository_id: *repo_id,
                number: pull_request.get_number() as i32,
                ..Default::default()
            },
        )?;
        pr_id_map.insert(pull_request.id, pr.id);
        pull_request.id = pr.id;
        pull_request.save(conn)?;
    }

    // Create or update reviews
    for review in &mut model.reviews {
        println!(
            "> Importing review for PR {} by @{}",
            review.id, review.username
        );

        let pr_id = pr_id_map.get(&review.pull_request_id).ok_or(
            ImportError::UnknownPullRequestIdError(review.pull_request_id),
        )?;
        let rvw = ReviewModel::get_or_create(
            conn,
            ReviewCreation {
                pull_request_id: *pr_id,
                username: &review.username,
                ..Default::default()
            },
        )?;

        // Update pull request if needed
        review.id = rvw.id;
        review.save(conn)?;
    }

    Ok(())
}