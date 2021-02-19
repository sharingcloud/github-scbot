//! Pull request commands.

use actix_rt::System;
use github_scbot_conf::Config;
use github_scbot_database::{
    establish_single_connection,
    models::{PullRequestModel, RepositoryModel},
};
use github_scbot_logic::pulls::synchronize_pull_request;
use github_scbot_logic::status::update_pull_request_status;

use super::errors::Result;

pub(crate) fn show_pull_request(config: &Config, repository_path: &str, number: u64) -> Result<()> {
    let conn = establish_single_connection(config)?;

    let pr = PullRequestModel::get_from_repository_path_and_number(
        &conn,
        &repository_path,
        number as i32,
    )?;
    println!(
        "Accessing pull request #{} on repository {}",
        number, repository_path
    );
    println!("{:#?}", pr);

    Ok(())
}

pub(crate) fn list_pull_requests(config: &Config, repository_path: &str) -> Result<()> {
    let conn = establish_single_connection(config)?;

    let prs = PullRequestModel::list_from_repository_path(&conn, &repository_path)?;
    if prs.is_empty() {
        println!("No PR found from repository '{}'.", repository_path);
    } else {
        for pr in prs {
            println!("- #{}: {}", pr.get_number(), pr.name);
        }
    }

    Ok(())
}

pub(crate) fn sync_pull_request(
    config: &Config,
    repository_path: String,
    number: u64,
) -> Result<()> {
    async fn sync(config: Config, repository_path: String, number: u64) -> Result<()> {
        let (owner, name) = RepositoryModel::extract_owner_and_name_from_path(&repository_path)?;

        let conn = establish_single_connection(&config)?;
        let (mut pr, sha) = synchronize_pull_request(&config, &conn, owner, name, number).await?;
        let repo = RepositoryModel::get_from_owner_and_name(&conn, owner, name)?;
        update_pull_request_status(&config, &conn, &repo, &mut pr, &sha).await?;

        println!(
            "Pull request #{} from {} updated from GitHub.",
            number, repository_path
        );
        Ok(())
    }

    let mut sys = System::new("sync");
    sys.block_on(sync(config.clone(), repository_path, number))
}
