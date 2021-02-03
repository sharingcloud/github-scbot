//! Shell module.

pub mod commands;

use std::path::PathBuf;

use github_scbot_core::configure_startup;
use github_scbot_server::server::run_bot_server;
use github_scbot_tui::run_tui;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
enum Command {
    /// Start bot server
    Server,

    /// Start TUI
    Ui,

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
        repository_path: String,
        /// Pull request number
        number: u64,
    },

    /// Synchronize pull request from upstream
    Sync {
        /// Repository path (e.g. 'MyOrganization/my-project')
        repository_path: String,
        // Pull request number
        number: u64,
    },

    /// List known pull requests for repository
    List { repository_path: String },
}

#[derive(StructOpt, Debug)]
enum RepositoryCommand {
    /// Set title validation regex
    SetTitleRegex {
        /// Repository path (e.g. 'MyOrganization/my-project')
        repository_path: String,
        /// Regex value
        value: String,
    },

    /// Show repository info
    Show {
        /// Repository path (e.g. 'MyOrganization/my-project')
        repository_path: String,
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

/// Initialize command line.
pub fn initialize_command_line() -> anyhow::Result<()> {
    // Prepare startup
    configure_startup()?;

    let opt = Opt::from_args();
    match opt.cmd {
        Command::Server => {
            run_bot_server()?;
        }
        Command::Ui => {
            run_tui()?;
        }
        Command::Export { output_file } => {
            commands::common::export_json(output_file)?;
        }
        Command::Import { input_file } => {
            commands::common::import_json(&input_file)?;
        }
        Command::Repository { cmd } => match cmd {
            RepositoryCommand::SetTitleRegex {
                repository_path,
                value,
            } => {
                commands::repository::set_pull_request_title_regex(&repository_path, &value)?;
            }
            RepositoryCommand::Show { repository_path } => {
                commands::repository::show_repository(&repository_path)?;
            }
            RepositoryCommand::List => {
                commands::repository::list_repositories()?;
            }
        },
        Command::PullRequest { cmd } => match cmd {
            PullRequestCommand::Show {
                repository_path,
                number,
            } => {
                commands::pull_request::show_pull_request(&repository_path, number)?;
            }
            PullRequestCommand::List { repository_path } => {
                commands::pull_request::list_pull_requests(&repository_path)?;
            }
            PullRequestCommand::Sync {
                repository_path,
                number,
            } => {
                commands::pull_request::sync_pull_request(repository_path, number)?;
            }
        },
    }

    Ok(())
}
