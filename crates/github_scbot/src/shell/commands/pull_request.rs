//! Pull request commands.

use actix_rt::System;
use github_scbot_api::pulls::get_pull_request;
use github_scbot_database::{
    errors::DatabaseError,
    establish_single_connection,
    models::{PullRequestCreation, PullRequestModel, RepositoryCreation, RepositoryModel},
};

use crate::errors::Result;

/// Show pull request data stored in database for a repository.
///
/// # Arguments
///
/// * `repository_path` - Repository path (<owner>/<name>)
/// * `number` - Pull request number
pub fn show_pull_request(repository_path: &str, number: u64) -> Result<()> {
    let conn = establish_single_connection()?;

    if let Some((pr, _repo)) = PullRequestModel::get_from_repository_path_and_number(
        &conn,
        &repository_path,
        number as i32,
    )? {
        println!(
            "Accessing pull request #{} on repository {}",
            number, repository_path
        );
        println!("{:#?}", pr);
    } else {
        println!(
            "No PR found for number #{} and repository {}",
            number, repository_path
        );
    }

    Ok(())
}

/// List known pull requests from database for a repository.
///
/// # Arguments
///
/// * `repository_path` - Repository path (<owner>/<name>)
pub fn list_pull_requests(repository_path: &str) -> Result<()> {
    let conn = establish_single_connection()?;

    let prs = PullRequestModel::list_for_repository_path(&conn, &repository_path)?;
    if prs.is_empty() {
        println!("No PR found for repository {}", repository_path);
    } else {
        for (pr, _repo) in prs {
            println!("- #{}: {}", pr.get_number(), pr.name);
        }
    }

    Ok(())
}

/// Synchronize a pull request from GitHub.
///
/// # Arguments
///
/// * `repository_path` - Repository path (<owner>/<name>)
/// * `number` - Pull request number
pub fn sync_pull_request(repository_path: String, number: u64) -> Result<()> {
    async fn sync(repository_path: String, number: u64) -> Result<()> {
        let (owner, name) = RepositoryModel::extract_owner_and_name_from_path(&repository_path)?;
        let target_pr = get_pull_request(owner, name, number).await.map_err(|_e| {
            DatabaseError::UnknownPullRequestError(number, repository_path.clone())
        })?;

        let conn = establish_single_connection()?;
        let repository = RepositoryModel::get_or_create(
            &conn,
            RepositoryCreation {
                name: name.into(),
                owner: owner.into(),
                ..Default::default()
            },
        )?;

        PullRequestModel::get_or_create(
            &conn,
            PullRequestCreation {
                repository_id: repository.id,
                name: target_pr.title,
                number: number as i32,
                ..Default::default()
            },
        )?;

        Ok(())
    }

    let mut sys = System::new("sync");
    sys.block_on(sync(repository_path, number))
}
