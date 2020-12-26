//! Database import/export module

use std::{
    collections::HashMap,
    io::{Read, Write},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{
    errors::Result,
    models::{PullRequestCreation, RepositoryCreation},
};
use super::{
    models::{PullRequestModel, RepositoryModel},
    DbConn,
};

#[derive(Debug, Error)]
pub enum ImportError {
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),

    #[error("IO error on file {0:?}: {1}")]
    IOError(PathBuf, std::io::Error),

    #[error("Unknown repository ID in file: {0}")]
    UnknownRepositoryIdError(i32),
}

#[derive(Debug, Error)]
pub enum ExportError {
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),

    #[error("IO error on file {0:?}: {1}")]
    IOError(PathBuf, std::io::Error),
}

#[derive(Debug, Deserialize, Serialize)]
struct ImportExportModel {
    repositories: Vec<RepositoryModel>,
    pull_requests: Vec<PullRequestModel>,
}

pub fn export_models_to_json<W>(conn: &DbConn, writer: &mut W) -> Result<()>
where
    W: Write,
{
    let model = ImportExportModel {
        repositories: RepositoryModel::list(conn)?,
        pull_requests: PullRequestModel::list(conn)?,
    };

    serde_json::to_writer_pretty(writer, &model).map_err(ExportError::SerdeError)?;

    Ok(())
}

pub fn import_models_from_json<R>(conn: &DbConn, reader: R) -> Result<()>
where
    R: Read,
{
    let model: ImportExportModel =
        serde_json::from_reader(reader).map_err(ImportError::SerdeError)?;

    // Create or update repositories
    let mut repo_id_map = HashMap::new();
    for repository in &model.repositories {
        println!(
            "> Importing repository {}/{}",
            repository.owner, repository.name
        );

        let mut repo = RepositoryModel::get_or_create(
            conn,
            &RepositoryCreation {
                owner: &repository.owner,
                name: &repository.name,
            },
        )?;
        repo_id_map.insert(repository.id, repo.id);
        repo.update_from_instance(conn, repository)?;
    }

    // Create or update pull requests
    for pull_request in &model.pull_requests {
        let repo_id = repo_id_map.get(&pull_request.repository_id).ok_or(
            ImportError::UnknownRepositoryIdError(pull_request.repository_id),
        )?;
        let mut pr = PullRequestModel::get_or_create(
            conn,
            &PullRequestCreation {
                repository_id: *repo_id,
                number: pull_request.number,
                ..PullRequestCreation::default()
            },
        )?;

        // Update pull request if needed
        pr.update_from_instance(conn, pull_request)?;
    }

    Ok(())
}
