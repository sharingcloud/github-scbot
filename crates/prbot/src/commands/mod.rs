//! Commands.

use std::{io::Write, sync::Arc};

use async_trait::async_trait;
use clap::Subcommand;
use prbot_config::Config;
use prbot_core::{CoreContext, CoreModule};
use prbot_database_interface::DbService;
use prbot_ghapi_interface::ApiService;
use prbot_lock_interface::LockService;
use tokio::sync::RwLock;

use self::{
    auth::AuthCommand, debug::DebugCommand, export::ExportCommand, import::ImportCommand,
    pull_request::PullRequestCommand, repository::RepositoryCommand, server::ServerCommand,
    ui::UiCommand, utils::UtilsCommand,
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
mod utils;

pub(crate) struct CommandContext {
    pub config: Config,
    pub db_service: Box<dyn DbService + Send + Sync>,
    pub api_service: Box<dyn ApiService + Send + Sync>,
    pub lock_service: Box<dyn LockService + Send + Sync>,
    pub core_module: CoreModule,
    pub writer: Arc<RwLock<dyn Write + Send + Sync>>,
}

impl CommandContext {
    pub fn as_core_context(&self) -> CoreContext {
        CoreContext {
            config: &self.config,
            core_module: &self.core_module,
            api_service: self.api_service.as_ref(),
            db_service: self.db_service.as_ref(),
            lock_service: self.lock_service.as_ref(),
        }
    }
}

#[async_trait]
pub(crate) trait Command {
    async fn execute(self, ctx: CommandContext) -> Result<()>;
}

/// Command
#[derive(Subcommand)]
pub(crate) enum SubCommand {
    Server(ServerCommand),
    Ui(UiCommand),
    Export(ExportCommand),
    Import(ImportCommand),
    Utils(UtilsCommand),
    PullRequests(PullRequestCommand),
    Repositories(RepositoryCommand),
    Auth(AuthCommand),
    Debug(DebugCommand),
}

#[async_trait]
impl Command for SubCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        match self {
            Self::Server(sub) => sub.execute(ctx).await,
            Self::Ui(sub) => sub.execute(ctx).await,
            Self::Export(sub) => sub.execute(ctx).await,
            Self::Import(sub) => sub.execute(ctx).await,
            Self::Utils(sub) => sub.execute(ctx).await,
            Self::PullRequests(sub) => sub.execute(ctx).await,
            Self::Auth(sub) => sub.execute(ctx).await,
            Self::Repositories(sub) => sub.execute(ctx).await,
            Self::Debug(sub) => sub.execute(ctx).await,
        }
    }
}
