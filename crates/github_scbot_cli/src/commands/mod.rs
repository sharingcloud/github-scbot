//! Commands.

use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_api::adapter::IAPIAdapter;
use github_scbot_conf::Config;
use github_scbot_database::{models::IDatabaseAdapter, DbPool};
use github_scbot_redis::IRedisAdapter;
use stable_eyre::eyre::Result;

use self::{
    auth::AuthCommand, debug::DebugCommand, export::ExportCommand, history::HistoryCommand,
    import::ImportCommand, pull_request::PullRequestCommand, repository::RepositoryCommand,
    server::ServerCommand, ui::UiCommand,
};

mod auth;
mod debug;
mod export;
mod history;
mod import;
mod pull_request;
mod repository;
mod server;
mod ui;

pub(crate) struct CommandContext<'a> {
    pub config: Config,
    pub pool: &'a DbPool,
    pub db_adapter: &'a dyn IDatabaseAdapter,
    pub api_adapter: &'a dyn IAPIAdapter,
    pub redis_adapter: &'a dyn IRedisAdapter,
    pub no_input: bool,
}

#[async_trait(?Send)]
pub(crate) trait Command {
    async fn execute<'a>(self, ctx: CommandContext<'a>) -> Result<()>;
}

/// Command
#[derive(FromArgs)]
#[argh(subcommand)]
pub(crate) enum SubCommand {
    Server(ServerCommand),
    Ui(UiCommand),
    Export(ExportCommand),
    Import(ImportCommand),
    PullRequest(PullRequestCommand),
    Repository(RepositoryCommand),
    Auth(AuthCommand),
    History(HistoryCommand),
    Debug(DebugCommand),
}

#[async_trait(?Send)]
impl Command for SubCommand {
    async fn execute<'a>(self, ctx: CommandContext<'a>) -> Result<()> {
        match self {
            Self::Server(sub) => sub.execute(ctx).await,
            Self::Ui(sub) => sub.execute(ctx).await,
            Self::Export(sub) => sub.execute(ctx).await,
            Self::Import(sub) => sub.execute(ctx).await,
            Self::PullRequest(sub) => sub.execute(ctx).await,
            Self::Auth(sub) => sub.execute(ctx).await,
            Self::Repository(sub) => sub.execute(ctx).await,
            Self::History(sub) => sub.execute(ctx).await,
            Self::Debug(sub) => sub.execute(ctx).await,
        }
    }
}
