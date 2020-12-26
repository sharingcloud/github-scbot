//! Repository commands

use crate::{
    database::{establish_single_connection, models::RepositoryModel},
    errors::Result,
};

pub fn command_set_title_regex(repo_path: &str, value: &str) -> Result<()> {
    let conn = establish_single_connection()?;

    if let Some(mut repo) = RepositoryModel::get_from_path(&conn, &repo_path)? {
        println!("Accessing repository {}", repo_path);
        println!("Setting value '{}' as PR title validation regex", value);
        repo.update_title_pr_validation_regex(&conn, &value)?;
    } else {
        eprintln!("Unknown repository {}.", repo_path);
    }

    Ok(())
}

pub fn command_show(repo_path: &str) -> Result<()> {
    let conn = establish_single_connection()?;

    if let Some(repo) = RepositoryModel::get_from_path(&conn, &repo_path)? {
        println!("Accessing repository {}", repo_path);
        println!("{:#?}", repo);
    } else {
        eprintln!("Unknown repository {}.", repo_path);
    }

    Ok(())
}

pub fn command_list() -> Result<()> {
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
