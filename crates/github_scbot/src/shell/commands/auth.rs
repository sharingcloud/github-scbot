//! Auth commands.

use github_scbot_core::Config;
use github_scbot_database::{
    establish_single_connection,
    models::{ExternalAccountModel, ExternalAccountRightModel, RepositoryModel},
};

use super::errors::Result;

pub(crate) fn create_external_account(config: &Config, username: &str) -> Result<()> {
    let conn = establish_single_connection(&config)?;

    ExternalAccountModel::create(&conn, username)?;
    println!("External account '{}' created.", username);

    Ok(())
}

pub(crate) fn list_external_accounts(config: &Config) -> Result<()> {
    let conn = establish_single_connection(&config)?;
    let accounts = ExternalAccountModel::list(&conn)?;
    if accounts.is_empty() {
        println!("No external account found.");
    } else {
        println!("External accounts:");
        for account in accounts {
            println!("- {}", account.username);
        }
    }

    Ok(())
}

pub(crate) fn remove_external_account(config: &Config, username: &str) -> Result<()> {
    let conn = establish_single_connection(&config)?;
    let account = ExternalAccountModel::get_from_username(&conn, username)?;
    account.remove(&conn)?;

    println!("External account '{}' removed.", username);

    Ok(())
}

pub(crate) fn create_external_token(config: &Config, username: &str) -> Result<()> {
    let conn = establish_single_connection(&config)?;
    let account = ExternalAccountModel::get_from_username(&conn, username)?;
    println!("{}", account.generate_access_token()?);

    Ok(())
}

pub(crate) fn add_account_right(
    config: &Config,
    username: &str,
    repository_path: &str,
) -> Result<()> {
    let conn = establish_single_connection(&config)?;
    let repo = RepositoryModel::get_from_path(&conn, repository_path)?;
    let account = ExternalAccountModel::get_from_username(&conn, username)?;

    ExternalAccountRightModel::add_right(&conn, &account.username, repo.id)?;
    println!(
        "Right added to repository '{}' for account '{}'.",
        repository_path, username
    );

    Ok(())
}

pub(crate) fn remove_account_right(
    config: &Config,
    username: &str,
    repository_path: &str,
) -> Result<()> {
    let conn = establish_single_connection(&config)?;
    let repo = RepositoryModel::get_from_path(&conn, repository_path)?;
    let account = ExternalAccountModel::get_from_username(&conn, username)?;

    ExternalAccountRightModel::remove_right(&conn, &account.username, repo.id)?;
    println!(
        "Right removed to repository '{}' for account '{}'.",
        repository_path, username
    );

    Ok(())
}

pub(crate) fn remove_account_rights(config: &Config, username: &str) -> Result<()> {
    let conn = establish_single_connection(&config)?;
    let account = ExternalAccountModel::get_from_username(&conn, username)?;

    ExternalAccountRightModel::remove_rights(&conn, &account.username)?;
    println!("All rights removed from account '{}'.", username);

    Ok(())
}

pub(crate) fn list_account_rights(config: &Config, username: &str) -> Result<()> {
    let conn = establish_single_connection(&config)?;

    let account = ExternalAccountModel::get_from_username(&conn, username)?;
    let rights = ExternalAccountRightModel::list_rights(&conn, &account.username)?;
    if rights.is_empty() {
        println!("No right found from account '{}'.", username);
    } else {
        println!("Rights from account '{}':", username);
        for right in rights {
            if let Ok(repo) = right.get_repository(&conn) {
                println!("- {}", repo.get_path());
            }
        }
    }

    Ok(())
}
