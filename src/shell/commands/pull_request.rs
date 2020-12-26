//! Pull request commands

use actix_web::rt;

use crate::errors::Result;
use crate::{
    api::pulls::get_pull_request,
    database::{
        errors::DatabaseError,
        establish_single_connection,
        models::{PullRequestCreation, PullRequestModel, RepositoryCreation, RepositoryModel},
    },
};

#[allow(clippy::cast_possible_truncation)]
pub fn command_show(repo_path: &str, number: u64) -> Result<()> {
    let conn = establish_single_connection()?;

    if let Some((pr, _repo)) =
        PullRequestModel::get_from_path_and_number(&conn, &repo_path, number as i32)?
    {
        println!(
            "Accessing pull request #{} on repository {}",
            number, repo_path
        );
        println!("{:#?}", pr);
    } else {
        println!(
            "No PR found for number #{} and repository {}",
            number, repo_path
        );
    }

    Ok(())
}

pub fn command_list(repo_path: &str) -> Result<()> {
    let conn = establish_single_connection()?;

    let prs = PullRequestModel::list_from_path(&conn, &repo_path)?;
    if prs.is_empty() {
        println!("No PR found for repository {}", repo_path);
    } else {
        for (pr, _repo) in prs {
            println!("- #{}: {}", pr.number, pr.name);
        }
    }

    Ok(())
}

pub fn command_sync(repo_path: String, number: u64) -> Result<()> {
    #[allow(clippy::cast_possible_truncation)]
    async fn sync(repo_path: String, number: u64) -> Result<()> {
        let (owner, name) = RepositoryModel::extract_name_from_path(&repo_path)?;
        let target_pr = get_pull_request(owner, name, number)
            .await
            .map_err(|_e| DatabaseError::UnknownPullRequestError(number, repo_path.clone()))?;

        let conn = establish_single_connection()?;
        let repository =
            RepositoryModel::get_or_create(&conn, &RepositoryCreation { name, owner })?;

        PullRequestModel::get_or_create(
            &conn,
            &PullRequestCreation {
                repository_id: repository.id,
                name: &target_pr.title,
                number: number as i32,
                ..PullRequestCreation::default()
            },
        )?;

        Ok(())
    }

    let mut sys = rt::System::new("sync");
    sys.block_on(sync(repo_path, number))
}
