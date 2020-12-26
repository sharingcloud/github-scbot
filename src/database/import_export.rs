//! Database import/export module

use serde::{Deserialize, Serialize};

use std::{
    collections::HashMap,
    io::{Read, Write},
};

use super::{
    errors::DatabaseError,
    models::{PullRequestCreation, RepositoryCreation},
};
use super::{
    models::{PullRequestModel, RepositoryModel},
    DbConn,
};

#[derive(Debug, Deserialize, Serialize)]
struct ImportExportModel {
    repositories: Vec<RepositoryModel>,
    pull_requests: Vec<PullRequestModel>,
}

pub fn export_models_to_json<W>(conn: &DbConn, writer: &mut W) -> Result<(), DatabaseError>
where
    W: Write,
{
    let model = ImportExportModel {
        repositories: RepositoryModel::list(conn)?,
        pull_requests: PullRequestModel::list(conn)?,
    };

    serde_json::to_writer_pretty(writer, &model)
        .map_err(|e| DatabaseError::ExportError(format!("Error on serialization: {}", e)))
}

pub fn import_models_from_json<R>(conn: &DbConn, reader: R) -> Result<(), DatabaseError>
where
    R: Read,
{
    let model: ImportExportModel = serde_json::from_reader(reader)
        .map_err(|e| DatabaseError::ImportError(format!("Error on deserialization: {}", e)))?;

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
        let repo_id = repo_id_map
            .get(&pull_request.repository_id)
            .ok_or_else(|| {
                DatabaseError::ImportError(format!(
                    "Unknown repository ID {0}",
                    pull_request.repository_id
                ))
            })?;
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
