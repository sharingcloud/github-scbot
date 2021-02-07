//! Pull request commands.

use actix_rt::System;
use anyhow::Result;
use github_scbot_core::Config;
use github_scbot_database::{
    establish_single_connection,
    models::{PullRequestModel, RepositoryModel},
};
use github_scbot_logic::pulls::synchronize_pull_request;

/// Show pull request data stored in database for a repository.
///
/// # Arguments
///
/// * `config` - Bot configuration
/// * `repository_path` - Repository path (<owner>/<name>)
/// * `number` - Pull request number
pub fn show_pull_request(config: &Config, repository_path: &str, number: u64) -> Result<()> {
    let conn = establish_single_connection(config)?;

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
/// * `config` - Bot configuration
/// * `repository_path` - Repository path (<owner>/<name>)
pub fn list_pull_requests(config: &Config, repository_path: &str) -> Result<()> {
    let conn = establish_single_connection(config)?;

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
/// * `config` - Bot configuration
/// * `repository_path` - Repository path (<owner>/<name>)
/// * `number` - Pull request number
pub fn sync_pull_request(config: &Config, repository_path: String, number: u64) -> Result<()> {
    async fn sync(config: Config, repository_path: String, number: u64) -> Result<()> {
        let (owner, name) = RepositoryModel::extract_owner_and_name_from_path(&repository_path)?;

        let conn = establish_single_connection(&config)?;
        synchronize_pull_request(&config, &conn, owner, name, number).await?;

        println!(
            "Pull request #{} from {} updated from GitHub.",
            number, repository_path
        );
        Ok(())
    }

    let mut sys = System::new("sync");
    sys.block_on(sync(config.clone(), repository_path, number))
}
