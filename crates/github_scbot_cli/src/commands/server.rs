use std::io::Write;

use crate::errors::ServerSnafu;
use crate::Result;
use async_trait::async_trait;
use clap::Parser;
use github_scbot_server::server::{run_bot_server, AppContext};
use snafu::ResultExt;

use super::{Command, CommandContext};

/// start server.
#[derive(Parser)]
pub(crate) struct ServerCommand;

#[async_trait(?Send)]
impl Command for ServerCommand {
    async fn execute<W: Write>(self, ctx: CommandContext<W>) -> Result<()> {
        let context = AppContext::new_with_adapters(
            ctx.config,
            ctx.db_adapter,
            ctx.api_adapter,
            ctx.redis_adapter,
        );

        run_bot_server(context).await.context(ServerSnafu)
    }
}
