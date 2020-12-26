//! Shell module

mod commands;

use std::path::PathBuf;

use structopt::StructOpt;

use crate::core::configure_startup;
use crate::server::run_bot_server;

#[derive(StructOpt, Debug)]
enum Command {
    /// Start bot server
    Server,

    /// Export data
    Export {
        /// Output file, stdout if not precised
        output_file: Option<PathBuf>,
    },

    /// Import data
    Import {
        /// Input file
        input_file: PathBuf,
    },

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
        Command::Export { output_file } => {
            commands::common::export_json(output_file)?;
        }
        Command::Import { input_file } => {
            commands::common::import_json(&input_file)?;
        }
        Command::Repository { cmd } => match cmd {
            RepositoryCommand::SetTitleRegex { repo_path, value } => {
                commands::repository::command_set_title_regex(&repo_path, &value)?;
            }
            RepositoryCommand::Show { repo_path } => {
                commands::repository::command_show(&repo_path)?;
            }
            RepositoryCommand::List => {
                commands::repository::command_list()?;
            }
        },
        Command::PullRequest { cmd } => match cmd {
            PullRequestCommand::Show { repo_path, number } => {
                commands::pull_request::command_show(&repo_path, number)?;
            }
            PullRequestCommand::List { repo_path } => {
                commands::pull_request::command_list(&repo_path)?;
            }
            PullRequestCommand::Sync { repo_path, number } => {
                commands::pull_request::command_sync(repo_path, number)?;
            }
        },
    }

    Ok(())
}
