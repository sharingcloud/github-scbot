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
    models::{PullRequestCreation, RepositoryCreation, ReviewCreation, ReviewModel},
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

    #[error("Unknown pull request ID in file: {0}")]
    UnknownPullRequestIdError(i32),
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
    reviews: Vec<ReviewModel>,
}

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

pub fn import_models_from_json<R>(conn: &DbConn, reader: R) -> Result<()>
where
    R: Read,
{
    let model: ImportExportModel =
        serde_json::from_reader(reader).map_err(ImportError::SerdeError)?;

    let mut repo_id_map = HashMap::new();
    let mut pr_id_map = HashMap::new();

    // Create or update repositories
    for repository in &model.repositories {
        println!(
            "> Importing repository {}/{}",
            repository.owner, repository.name
        );

        let mut repo = RepositoryModel::get_or_create(
            conn,
            RepositoryCreation {
                name: &repository.name,
                owner: &repository.owner,
            },
        )?;
        repo_id_map.insert(repository.id, repo.id);
        repo.update_from_instance(conn, repository)?;
    }

    // Create or update pull requests
    for pull_request in &model.pull_requests {
        println!("> Importing pull request #{}", pull_request.number);

        let repo_id = repo_id_map.get(&pull_request.repository_id).ok_or(
            ImportError::UnknownRepositoryIdError(pull_request.repository_id),
        )?;
        let mut pr = PullRequestModel::get_or_create(
            conn,
            PullRequestCreation {
                repository_id: *repo_id,
                number: pull_request.number,
                ..PullRequestCreation::default()
            },
        )?;
        pr_id_map.insert(pull_request.id, pr.id);
        pr.update_from_instance(conn, pull_request)?;
    }

    // Create or update reviews
    for review in &model.reviews {
        println!(
            "> Importing review for PR {} by @{}",
            review.id, review.username
        );

        let pr_id = pr_id_map.get(&review.pull_request_id).ok_or(
            ImportError::UnknownPullRequestIdError(review.pull_request_id),
        )?;
        let mut rvw = ReviewModel::get_or_create(
            conn,
            ReviewCreation {
                pull_request_id: *pr_id,
                username: &review.username,
                ..Default::default()
            },
        )?;

        // Update pull request if needed
        rvw.update_from_instance(conn, review)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{export_models_to_json, import_models_from_json};
    use crate::{
        database::{
            establish_single_connection,
            models::{
                PullRequestCreation, PullRequestModel, RepositoryCreation, RepositoryModel,
                ReviewCreation, ReviewModel,
            },
        },
        types::PullRequestReviewState,
        utils::test_init,
    };

    #[test]
    fn test_export_models_to_json() {
        test_init();

        let conn = establish_single_connection().unwrap();
        let repo = RepositoryModel::create(
            &conn,
            RepositoryCreation {
                name: "TestRepo",
                owner: "me",
            },
        )
        .unwrap();

        let pr = PullRequestModel::create(
            &conn,
            PullRequestCreation {
                repository_id: repo.id,
                number: 1234,
                name: "Toto",
                ..Default::default()
            },
        )
        .unwrap();

        ReviewModel::create(
            &conn,
            ReviewCreation {
                pull_request_id: pr.id,
                required: true,
                state: PullRequestReviewState::Commented.to_string(),
                username: "toto",
            },
        )
        .unwrap();

        let mut buffer = Vec::new();
        export_models_to_json(&conn, &mut buffer).unwrap();

        let buffer_string = String::from_utf8(buffer).unwrap();
        assert!(buffer_string.contains(r#""name": "TestRepo""#));
        assert!(buffer_string.contains(r#""number": 1234"#));
        assert!(buffer_string.contains(r#""username": "toto"#));
    }

    #[test]
    fn test_import_models_from_json() {
        test_init();

        let conn = establish_single_connection().unwrap();
        let repo = RepositoryModel::create(
            &conn,
            RepositoryCreation {
                name: "TestRepo",
                owner: "me",
            },
        )
        .unwrap();

        PullRequestModel::create(
            &conn,
            PullRequestCreation {
                repository_id: repo.id,
                number: 1234,
                name: "Toto",
                ..Default::default()
            },
        )
        .unwrap();

        let sample = r#"
            {
                "repositories": [
                    {
                        "id": 1,
                        "name": "TestRepo",
                        "owner": "me",
                        "pr_title_validation_regex": "[a-z]*",
                        "default_needed_reviewers_count": 2
                    },
                    {
                        "id": 2,
                        "name": "AnotherRepo",
                        "owner": "me",
                        "pr_title_validation_regex": "",
                        "default_needed_reviewers_count": 3
                    }
                ],
                "pull_requests": [
                    {
                        "id": 1,
                        "repository_id": 1,
                        "number": 1234,
                        "name": "Tutu",
                        "automerge": false,
                        "step": "step/awaiting-review",
                        "check_status": null,
                        "status_comment_id": 1,
                        "qa_status": null,
                        "wip": false,
                        "needed_reviewers_count": 2
                    },
                    {
                        "id": 2,
                        "repository_id": 1,
                        "number": 1235,
                        "name": "Tata",
                        "automerge": true,
                        "step": "step/wip",
                        "check_status": "pass",
                        "status_comment_id": 0,
                        "qa_status": "pass",
                        "wip": true,
                        "needed_reviewers_count": 2
                    }
                ],
                "reviews": [
                    {
                        "id": 1,
                        "pull_request_id": 1,
                        "username": "tutu",
                        "state": "commented",
                        "required": true
                    }
                ]
            }
        "#;

        import_models_from_json(&conn, sample.as_bytes()).unwrap();

        let rep_1 = RepositoryModel::get_from_name(&conn, "TestRepo", "me").unwrap();
        let rep_2 = RepositoryModel::get_from_name(&conn, "AnotherRepo", "me").unwrap();
        let pr_1 = PullRequestModel::get_from_number(&conn, rep_1.id, 1234).unwrap();
        let pr_2 = PullRequestModel::get_from_number(&conn, rep_1.id, 1235).unwrap();
        let review_1 =
            ReviewModel::get_from_pull_request_and_username(&conn, pr_1.id, "tutu").unwrap();

        assert_eq!(rep_1.pr_title_validation_regex, "[a-z]*");
        assert_eq!(rep_2.pr_title_validation_regex, "");
        assert_eq!(pr_1.name, "Tutu");
        assert_eq!(pr_1.automerge, false);
        assert_eq!(pr_2.name, "Tata");
        assert_eq!(pr_2.automerge, true);
        assert_eq!(review_1.required, true);
        assert_eq!(review_1.state_enum(), PullRequestReviewState::Commented);
    }
}
