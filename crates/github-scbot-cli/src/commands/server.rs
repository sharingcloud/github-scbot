use std::io::Write;

use async_trait::async_trait;
use clap::Parser;
use github_scbot_server::server::{run_bot_server, AppContext};

use super::{Command, CommandContext};
use crate::Result;

/// Start server
#[derive(Parser)]
pub(crate) struct ServerCommand;

#[async_trait(?Send)]
impl Command for ServerCommand {
    async fn execute<W: Write>(self, ctx: CommandContext<W>) -> Result<()> {
        let context = AppContext::new_with_adapters(
            ctx.config,
            ctx.db_service,
            ctx.api_service,
            ctx.lock_service,
        );

        run_bot_server(context).await.map_err(Into::into)
    }
}
