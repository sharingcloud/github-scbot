//! Auth commands.

use anyhow::Result;
use github_scbot_core::Config;
use github_scbot_database::{establish_single_connection, models::ExternalAccountModel};

pub(crate) fn create_external_account(config: &Config, username: &str) -> Result<()> {
    let conn = establish_single_connection(&config)?;

    ExternalAccountModel::create(&conn, username)?;
    println!("External account '{}' created.", username);

    Ok(())
}

pub(crate) fn create_external_token(config: &Config, username: &str) -> Result<()> {
    let conn = establish_single_connection(&config)?;

    match ExternalAccountModel::get_from_username(&conn, username) {
        Some(account) => println!("{}", account.generate_access_token()?),
        None => eprintln!("External account '{}' does not exist.", username),
    }

    Ok(())
}
