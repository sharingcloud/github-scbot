//! Shell module

use std::convert::TryInto;

use actix_web::rt;
use structopt::StructOpt;

use crate::{
    core::configure_startup,
    database::models::{PullRequestCreation, RepositoryCreation},
};

use crate::api::pulls::get_pull_request;
use crate::database::{establish_single_connection, models::PullRequestModel};
use crate::errors::{BotError, Result};
use crate::{database::models::RepositoryModel, server::run_bot_server};

#[derive(StructOpt, Debug)]
enum Command {
    /// Start bot server
    Server,

    /// Configure pull request
    PullRequest {
        /// Pull request configuration command
        #[structopt(subcommand)]
        cmd: PullRequestCommand,
    },

    /// Configure repository
    Repository {
        /// Repository configuration command
        #[structopt(subcommand)]
        cmd: RepositoryCommand,
    },
}

#[derive(StructOpt, Debug)]
enum PullRequestCommand {
    /// Show pull request stored data
    Show {
        /// Repository path (e.g. 'MyOrganization/my-project')
        repo_path: String,
        /// Pull request number
        number: u64,
    },

    /// Synchronize pull request from upstream
    Sync {
        /// Repository path (e.g. 'MyOrganization/my-project')
        repo_path: String,
        // Pull request number
        number: u64,
    },

    /// List known pull requests for repository
    List { repo_path: String },
}

#[derive(StructOpt, Debug)]
enum RepositoryCommand {
    /// Set title validation regex
    SetTitleRegex {
        /// Repository path (e.g. 'MyOrganization/my-project')
        repo_path: String,
        /// Regex value
        value: String,
    },

    /// Show repository info
    Show {
        /// Repository path (e.g. 'MyOrganization/my-project')
        repo_path: String,
    },

    /// List known repositories
    List,
}

#[derive(StructOpt, Debug)]
struct Opt {
    /// Activate verbose mode
    #[structopt(short, long)]
    verbose: bool,

    #[structopt(subcommand)]
    cmd: Command,
}

/// Initialize command line
///
/// # Errors
///
/// - Startup error
///
pub fn initialize_command_line() -> anyhow::Result<()> {
    // Prepare startup
    configure_startup()?;

    let opt = Opt::from_args();
    match opt.cmd {
        Command::Server => {
            run_bot_server()?;
        }
        Command::Repository { cmd } => match cmd {
            RepositoryCommand::SetTitleRegex { repo_path, value } => {
                let conn = establish_single_connection()?;

                if let Some(mut repo) = RepositoryModel::get_from_path(&conn, &repo_path)? {
                    println!("Accessing repository {}", repo_path);
                    println!("Setting value '{}' as PR title validation regex", value);
                    repo.update_title_pr_validation_regex(&conn, &value)?;
                } else {
                    eprintln!("Unknown repository {}.", repo_path);
                }
            }
            RepositoryCommand::Show { repo_path } => {
                let conn = establish_single_connection()?;

                if let Some(repo) = RepositoryModel::get_from_path(&conn, &repo_path)? {
                    println!("Accessing repository {}", repo_path);
                    println!("{:#?}", repo);
                } else {
                    eprintln!("Unknown repository {}.", repo_path);
                }
            }
            RepositoryCommand::List => {
                let conn = establish_single_connection()?;

                let repos = RepositoryModel::list(&conn)?;
                if repos.is_empty() {
                    println!("No repository known.");
                } else {
                    for repo in repos {
                        println!("- {}/{}", repo.owner, repo.name);
                    }
                }
            }
        },
        Command::PullRequest { cmd } => match cmd {
            PullRequestCommand::Show { repo_path, number } => {
                let conn = establish_single_connection()?;

                if let Some((pr, _repo)) = PullRequestModel::get_from_path_and_number(
                    &conn,
                    &repo_path,
                    number.try_into()?,
                )? {
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
            }
            PullRequestCommand::List { repo_path } => {
                let conn = establish_single_connection()?;

                let prs = PullRequestModel::list_from_path(&conn, &repo_path)?;
                if prs.is_empty() {
                    println!("No PR found for repository {}", repo_path);
                } else {
                    for (pr, _repo) in prs {
                        println!("- #{}: {}", pr.number, pr.name);
                    }
                }
            }
            PullRequestCommand::Sync { repo_path, number } => {
                async fn sync(repo_path: String, number: u64) -> Result<()> {
                    let (owner, name) = RepositoryModel::extract_name_from_path(&repo_path)?;
                    let target_pr = get_pull_request(owner, name, number)
                        .await
                        .map_err(|_e| BotError::UnknownPullRequest(repo_path.clone(), number))?;

                    let conn = establish_single_connection()?;
                    let repository =
                        RepositoryModel::get_or_create(&conn, &RepositoryCreation { name, owner })?;

                    PullRequestModel::get_or_create(
                        &conn,
                        &PullRequestCreation {
                            repository_id: repository.id,
                            name: &target_pr.title,
                            number: number.try_into()?,
                            ..PullRequestCreation::default()
                        },
                    )?;

                    Ok(())
                }

                let mut sys = rt::System::new("sync");
                sys.block_on(sync(repo_path, number))?;
            }
        },
    }

    Ok(())
}
