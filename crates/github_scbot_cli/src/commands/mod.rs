//! Commands.

use std::io::Write;

use crate::Result;
use async_trait::async_trait;
use clap::Subcommand;
use github_scbot_conf::Config;
use github_scbot_database2::DbService;
use github_scbot_ghapi::adapter::ApiService;
use github_scbot_redis::RedisService;

use self::{
    auth::AuthCommand, debug::DebugCommand, export::ExportCommand, import::ImportCommand,
    pull_request::PullRequestCommand, repository::RepositoryCommand, server::ServerCommand,
    ui::UiCommand,
};
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
    pub db_adapter: Box<dyn DbService>,
    pub api_adapter: Box<dyn ApiService>,
    pub redis_adapter: Box<dyn RedisService>,
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
