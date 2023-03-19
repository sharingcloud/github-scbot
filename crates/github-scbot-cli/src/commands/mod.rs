//! Commands.

use std::io::Write;

use async_trait::async_trait;
use clap::Subcommand;
use github_scbot_config::Config;
use github_scbot_database_interface::DbService;
use github_scbot_ghapi_interface::ApiService;
use github_scbot_lock_interface::LockService;

use self::{
    auth::AuthCommand, debug::DebugCommand, export::ExportCommand, import::ImportCommand,
    pull_request::PullRequestCommand, repository::RepositoryCommand, server::ServerCommand,
    ui::UiCommand,
};
use crate::Result;

mod auth;
mod debug;
mod export;
mod import;
mod pull_request;
mod repository;
mod server;
mod ui;

pub(crate) struct CommandContext<W: Write> {
    pub config: Config,
    pub db_service: Box<dyn DbService>,
    pub api_service: Box<dyn ApiService>,
    pub lock_service: Box<dyn LockService>,
    pub writer: W,
}

#[async_trait(?Send)]
pub(crate) trait Command {
    async fn execute<W: Write>(self, ctx: CommandContext<W>) -> Result<()>;
}

/// Command
#[derive(Subcommand)]
pub(crate) enum SubCommand {
    Server(ServerCommand),
    Ui(UiCommand),
    Export(ExportCommand),
    Import(ImportCommand),
    PullRequests(PullRequestCommand),
    Repositories(RepositoryCommand),
    Auth(AuthCommand),
    Debug(DebugCommand),
}

#[async_trait(?Send)]
impl Command for SubCommand {
    async fn execute<W: Write>(self, ctx: CommandContext<W>) -> Result<()> {
        match self {
            Self::Server(sub) => sub.execute(ctx).await,
            Self::Ui(sub) => sub.execute(ctx).await,
            Self::Export(sub) => sub.execute(ctx).await,
            Self::Import(sub) => sub.execute(ctx).await,
            Self::PullRequests(sub) => sub.execute(ctx).await,
            Self::Auth(sub) => sub.execute(ctx).await,
            Self::Repositories(sub) => sub.execute(ctx).await,
            Self::Debug(sub) => sub.execute(ctx).await,
        }
    }
}
