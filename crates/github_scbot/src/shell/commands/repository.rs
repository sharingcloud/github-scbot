//! Repository commands.

use github_scbot_database::{establish_single_connection, models::RepositoryModel};

use crate::errors::Result;

/// Set the pull request title validation regex for a repository.
///
/// # Arguments
///
/// * `repository_path` - Repository path (<owner>/<name>)
/// * `value` - Regex value
pub fn set_pull_request_title_regex(repository_path: &str, value: &str) -> Result<()> {
    let conn = establish_single_connection()?;

    if let Some(mut repo) = RepositoryModel::get_from_path(&conn, &repository_path)? {
        println!("Accessing repository {}", repository_path);
        println!("Setting value '{}' as PR title validation regex", value);
        repo.pr_title_validation_regex = value.to_owned();
        repo.save(&conn)?;
    } else {
        eprintln!("Unknown repository {}.", repository_path);
    }

    Ok(())
}

/// Show repository data stored in database.
///
/// # Arguments
///
/// * `repository_path` - Repository path (<owner>/<name>)
pub fn show_repository(repository_path: &str) -> Result<()> {
    let conn = establish_single_connection()?;

    if let Some(repo) = RepositoryModel::get_from_path(&conn, &repository_path)? {
        println!("Accessing repository {}", repository_path);
        println!("{:#?}", repo);
    } else {
        eprintln!("Unknown repository {}.", repository_path);
    }

    Ok(())
}

/// List known repositories from database.
pub fn list_repositories() -> Result<()> {
    let conn = establish_single_connection()?;

    let repos = RepositoryModel::list(&conn)?;
    if repos.is_empty() {
        println!("No repository known.");
    } else {
        for repo in repos {
            println!("- {}/{}", repo.owner, repo.name);
        }
    }

    Ok(())
}
