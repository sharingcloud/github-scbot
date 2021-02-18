//! Shell module.

pub mod commands;

use std::path::PathBuf;

use github_scbot_conf::configure_startup;
use github_scbot_server::server::run_bot_server;
use github_scbot_tui::run_tui;
use structopt::StructOpt;

#[derive(StructOpt)]
enum Command {
    /// Start bot server
    Server,

    /// Start TUI application
    Ui,

    /// Export all data
    Export {
        /// Output file, stdout if not precised
        output_file: Option<PathBuf>,
    },

    /// Import all data
    Import {
        /// Input file
        input_file: PathBuf,
    },

    /// Pull request management
    PullRequest {
        /// Pull request management command
        #[structopt(subcommand)]
        cmd: PullRequestCommand,
    },

    /// Repository management
    Repository {
        /// Repository management command
        #[structopt(subcommand)]
        cmd: RepositoryCommand,
    },

    /// Authentication management
    Auth {
        /// Authentication management command
        #[structopt(subcommand)]
        cmd: AuthCommand,
    },
}

#[derive(StructOpt)]
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

#[derive(StructOpt)]
enum RepositoryCommand {
    /// Set title validation regex
    SetTitleRegex {
        /// Repository path (e.g. `MyOrganization/my-project`)
        repository_path: String,
        /// Regex value
        value: String,
    },

    /// Show repository info
    Show {
        /// Repository path (e.g. `MyOrganization/my-project`)
        repository_path: String,
    },

    /// Set default reviewers count
    SetReviewersCount {
        /// Repository path (e.g. `MyOrganization/my-project`)
        repository_path: String,
        /// Reviewers count
        reviewers_count: u32,
    },

    /// Set merge rule
    SetMergeRule {
        /// Repository path (e.g. `MyOrganization/my-project`)
        repository_path: String,
        /// Base branch name
        base_branch: String,
        /// Head branch name
        head_branch: String,
        /// Strategy
        strategy: String,
    },

    /// Remove merge rule
    RemoveMergeRule {
        /// Repository path (e.g. `MyOrganization/my-project`)
        repository_path: String,
        /// Base branch name
        base_branch: String,
        /// Head branch name
        head_branch: String,
    },

    /// List merge rules
    ListMergeRules {
        /// Repository path (e.g. `MyOrganization/my-project`)
        repository_path: String,
    },

    /// Purge closed pull requests
    Purge {
        /// Repository path (e.g. `MyOrganization/my-project`)
        repository_path: String,
    },

    /// List known repositories
    List,
}

#[derive(StructOpt)]
enum AuthCommand {
    /// Create external account
    CreateExternalAccount {
        /// Account username
        username: String,
    },

    /// Create external token
    CreateExternalToken {
        /// Account username
        username: String,
    },

    /// Remove external account
    RemoveExternalAccount {
        /// Account username
        username: String,
    },

    /// List external accounts
    ListExternalAccounts,

    /// Add right to account
    AddAccountRight {
        /// Account username
        username: String,
        /// Repository path (e.g. `MyOrganization/my-project`)
        repository_path: String,
    },

    /// Remove right from account
    RemoveAccountRight {
        /// Account username
        username: String,
        /// Repository path (e.g. `MyOrganization/my-project`)
        repository_path: String,
    },

    /// Remove all rights from account
    RemoveAccountRights {
        /// Account username
        username: String,
    },

    /// List rights from account
    ListAccountRights {
        /// Account username
        username: String,
    },

    /// Add admin rights
    AddAdminRights {
        /// Account username
        username: String,
    },

    /// Remove admin rights
    RemoveAdminRights {
        /// Account username
        username: String,
    },

    /// List admin accounts
    ListAdminAccounts,
}

#[derive(StructOpt)]
struct Opt {
    #[structopt(subcommand)]
    cmd: Command,
}

/// Initialize command line.
pub fn initialize_command_line() -> anyhow::Result<()> {
    // Prepare startup
    let config = configure_startup()?;

    let opt = Opt::from_args();
    match opt.cmd {
        Command::Server => {
            run_bot_server(config)?;
        }
        Command::Ui => {
            run_tui()?;
        }
        Command::Export { output_file } => {
            commands::database::export_json(&config, output_file)?;
        }
        Command::Import { input_file } => {
            commands::database::import_json(&config, &input_file)?;
        }
        Command::Repository { cmd } => match cmd {
            RepositoryCommand::SetTitleRegex {
                repository_path,
                value,
            } => {
                commands::repository::set_pull_request_title_regex(
                    &config,
                    &repository_path,
                    &value,
                )?;
            }
            RepositoryCommand::SetReviewersCount {
                repository_path,
                reviewers_count,
            } => {
                commands::repository::set_reviewers_count(
                    &config,
                    &repository_path,
                    reviewers_count,
                )?;
            }
            RepositoryCommand::ListMergeRules { repository_path } => {
                commands::repository::list_merge_rules(&config, &repository_path)?;
            }
            RepositoryCommand::SetMergeRule {
                repository_path,
                base_branch,
                head_branch,
                strategy,
            } => {
                commands::repository::set_merge_rule(
                    &config,
                    &repository_path,
                    &base_branch,
                    &head_branch,
                    &strategy,
                )?;
            }
            RepositoryCommand::RemoveMergeRule {
                repository_path,
                base_branch,
                head_branch,
            } => {
                commands::repository::remove_merge_rule(
                    &config,
                    &repository_path,
                    &base_branch,
                    &head_branch,
                )?;
            }
            RepositoryCommand::Show { repository_path } => {
                commands::repository::show_repository(&config, &repository_path)?;
            }
            RepositoryCommand::List => {
                commands::repository::list_repositories(&config)?;
            }
            RepositoryCommand::Purge { repository_path } => {
                commands::repository::purge_pull_requests(&config, &repository_path)?;
            }
        },
        Command::PullRequest { cmd } => match cmd {
            PullRequestCommand::Show {
                repository_path,
                number,
            } => {
                commands::pulls::show_pull_request(&config, &repository_path, number)?;
            }
            PullRequestCommand::List { repository_path } => {
                commands::pulls::list_pull_requests(&config, &repository_path)?;
            }
            PullRequestCommand::Sync {
                repository_path,
                number,
            } => {
                commands::pulls::sync_pull_request(&config, repository_path, number)?;
            }
        },
        Command::Auth { cmd } => match cmd {
            AuthCommand::CreateExternalAccount { username } => {
                commands::auth::create_external_account(&config, &username)?;
            }
            AuthCommand::CreateExternalToken { username } => {
                commands::auth::create_external_token(&config, &username)?;
            }
            AuthCommand::RemoveExternalAccount { username } => {
                commands::auth::remove_external_account(&config, &username)?;
            }
            AuthCommand::ListExternalAccounts => {
                commands::auth::list_external_accounts(&config)?;
            }
            AuthCommand::AddAccountRight {
                username,
                repository_path,
            } => {
                commands::auth::add_account_right(&config, &username, &repository_path)?;
            }
            AuthCommand::RemoveAccountRight {
                username,
                repository_path,
            } => {
                commands::auth::remove_account_right(&config, &username, &repository_path)?;
            }
            AuthCommand::RemoveAccountRights { username } => {
                commands::auth::remove_account_rights(&config, &username)?;
            }
            AuthCommand::ListAccountRights { username } => {
                commands::auth::list_account_rights(&config, &username)?;
            }
            AuthCommand::AddAdminRights { username } => {
                commands::auth::add_admin_rights(&config, &username)?;
            }
            AuthCommand::RemoveAdminRights { username } => {
                commands::auth::remove_admin_rights(&config, &username)?;
            }
            AuthCommand::ListAdminAccounts => {
                commands::auth::list_admin_accounts(&config)?;
            }
        },
    }

    Ok(())
}
