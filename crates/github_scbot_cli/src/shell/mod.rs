//! Shell module.

pub mod commands;

use std::path::PathBuf;

use actix_rt::System;
use github_scbot_api::adapter::GithubAPIAdapter;
use github_scbot_conf::{configure_startup, Config};
use github_scbot_database::{
    establish_pool_connection,
    models::{DatabaseAdapter, IDatabaseAdapter},
    run_migrations, DbPool,
};
use github_scbot_redis::RedisAdapter;
use github_scbot_server::server::run_bot_server;
use github_scbot_tui::run_tui;
use stable_eyre::eyre;
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

    /// History management
    History {
        /// History management ocmmand
        #[structopt(subcommand)]
        cmd: HistoryCommand,
    },

    /// Debug commands
    Debug {
        /// Debug command
        #[structopt(subcommand)]
        cmd: DebugCommand,
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
    /// Add repository
    Add {
        /// Repository path (e.g. `MyOrganization/my-project`)
        repository_path: String,
    },

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

    /// Set manual interaction mode
    SetManualInteraction {
        /// Repository path (e.g. `MyOrganization/my-project`)
        repository_path: String,
        // Mode
        #[structopt(parse(try_from_str))]
        manual_interaction: bool,
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
enum HistoryCommand {
    /// List webhook events for repository
    ListWebhookEvents {
        /// Repository path (e.g. 'MyOrganization/my-project')
        repository_path: String,
    },

    /// Remove all webhook events
    RemoveWebhookEvents,
}

#[derive(StructOpt)]
enum DebugCommand {
    /// Send a test event to Sentry to troubleshoot connection issues
    TestSentry {
        /// Custom message. Default: "This is a test".
        message: Option<String>,
    },
}

#[derive(StructOpt)]
struct Opt {
    #[structopt(subcommand)]
    cmd: Command,

    #[structopt(short, long)]
    no_input: bool,
}

/// Initialize command line.
pub fn initialize_command_line() -> eyre::Result<()> {
    // Prepare startup
    let config = configure_startup()?;

    async fn sync(config: Config, cmd: Command, no_input: bool) -> eyre::Result<()> {
        let pool = establish_pool_connection(&config)?;
        run_migrations(&pool)?;

        let db_adapter = DatabaseAdapter::new(&pool);
        parse_args(config, &pool, &db_adapter, cmd, no_input).await
    }

    let opt = Opt::from_args();
    let mut sys = System::new("app");
    sys.block_on(sync(config, opt.cmd, opt.no_input))?;

    Ok(())
}

async fn parse_args(
    config: Config,
    pool: &DbPool,
    db_adapter: &dyn IDatabaseAdapter,
    cmd: Command,
    no_input: bool,
) -> eyre::Result<()> {
    match cmd {
        Command::Server => {
            run_bot_server(config, pool).await?;
        }
        Command::Ui => {
            run_tui(db_adapter).await?;
        }
        Command::Export { output_file } => {
            commands::database::export_json(db_adapter, output_file).await?;
        }
        Command::Import { input_file } => {
            commands::database::import_json(&config, db_adapter, &input_file).await?;
        }
        Command::Repository { cmd } => match cmd {
            RepositoryCommand::Add { repository_path } => {
                commands::repository::add_repository(&config, db_adapter, &repository_path).await?
            }
            RepositoryCommand::SetTitleRegex {
                repository_path,
                value,
            } => {
                commands::repository::set_pull_request_title_regex(
                    db_adapter,
                    &repository_path,
                    &value,
                )
                .await?
            }
            RepositoryCommand::SetReviewersCount {
                repository_path,
                reviewers_count,
            } => {
                commands::repository::set_reviewers_count(
                    db_adapter,
                    &repository_path,
                    reviewers_count,
                )
                .await?
            }
            RepositoryCommand::ListMergeRules { repository_path } => {
                commands::repository::list_merge_rules(db_adapter, &repository_path).await?
            }
            RepositoryCommand::SetMergeRule {
                repository_path,
                base_branch,
                head_branch,
                strategy,
            } => {
                commands::repository::set_merge_rule(
                    db_adapter,
                    &repository_path,
                    &base_branch,
                    &head_branch,
                    &strategy,
                )
                .await?
            }
            RepositoryCommand::RemoveMergeRule {
                repository_path,
                base_branch,
                head_branch,
            } => {
                commands::repository::remove_merge_rule(
                    db_adapter,
                    &repository_path,
                    &base_branch,
                    &head_branch,
                )
                .await?
            }
            RepositoryCommand::SetManualInteraction {
                repository_path,
                manual_interaction,
            } => {
                commands::repository::set_manual_interaction_mode(
                    db_adapter,
                    &repository_path,
                    manual_interaction,
                )
                .await?
            }
            RepositoryCommand::Show { repository_path } => {
                commands::repository::show_repository(db_adapter, &repository_path).await?
            }
            RepositoryCommand::List => commands::repository::list_repositories(db_adapter).await?,
            RepositoryCommand::Purge { repository_path } => {
                commands::repository::purge_pull_requests(db_adapter, &repository_path).await?
            }
        },
        Command::PullRequest { cmd } => match cmd {
            PullRequestCommand::Show {
                repository_path,
                number,
            } => {
                commands::pulls::show_pull_request(db_adapter, &repository_path, number).await?;
            }
            PullRequestCommand::List { repository_path } => {
                commands::pulls::list_pull_requests(db_adapter, &repository_path).await?;
            }
            PullRequestCommand::Sync {
                repository_path,
                number,
            } => {
                let api_adapter = GithubAPIAdapter::new(&config).await?;
                let redis_adapter = RedisAdapter::new(&config.redis_address);

                commands::pulls::sync_pull_request(
                    &config,
                    &api_adapter,
                    db_adapter,
                    &redis_adapter,
                    repository_path,
                    number,
                )
                .await?;
            }
        },
        Command::Auth { cmd } => match cmd {
            AuthCommand::CreateExternalAccount { username } => {
                commands::auth::create_external_account(db_adapter, &username).await?
            }
            AuthCommand::CreateExternalToken { username } => {
                commands::auth::create_external_token(db_adapter, &username).await?
            }
            AuthCommand::RemoveExternalAccount { username } => {
                commands::auth::remove_external_account(db_adapter, &username).await?
            }
            AuthCommand::ListExternalAccounts => {
                commands::auth::list_external_accounts(db_adapter).await?
            }
            AuthCommand::AddAccountRight {
                username,
                repository_path,
            } => commands::auth::add_account_right(db_adapter, &username, &repository_path).await?,
            AuthCommand::RemoveAccountRight {
                username,
                repository_path,
            } => {
                commands::auth::remove_account_right(db_adapter, &username, &repository_path)
                    .await?
            }
            AuthCommand::RemoveAccountRights { username } => {
                commands::auth::remove_account_rights(db_adapter, &username).await?
            }
            AuthCommand::ListAccountRights { username } => {
                commands::auth::list_account_rights(db_adapter, &username).await?
            }
            AuthCommand::AddAdminRights { username } => {
                commands::auth::add_admin_rights(db_adapter, &username).await?
            }
            AuthCommand::RemoveAdminRights { username } => {
                commands::auth::remove_admin_rights(db_adapter, &username).await?
            }
            AuthCommand::ListAdminAccounts => {
                commands::auth::list_admin_accounts(db_adapter).await?
            }
        },
        Command::History { cmd } => match cmd {
            HistoryCommand::ListWebhookEvents { repository_path } => {
                commands::history::list_webhook_events_from_repository(db_adapter, &repository_path)
                    .await?
            }
            HistoryCommand::RemoveWebhookEvents => {
                commands::history::remove_webhook_events(db_adapter, no_input).await?
            }
        },
        Command::Debug { cmd } => match cmd {
            DebugCommand::TestSentry { message } => {
                commands::debug::send_test_event_to_sentry(&config, message).await?;
            }
        },
    }

    Ok(())
}
