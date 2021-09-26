use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_server::server::run_bot_server;
use stable_eyre::eyre::Result;

use super::{Command, CommandContext};

/// start server.
#[derive(FromArgs)]
#[argh(subcommand, name = "server")]
pub(crate) struct ServerCommand {}

#[async_trait(?Send)]
impl Command for ServerCommand {
    async fn execute<'a>(self, ctx: CommandContext<'a>) -> Result<()> {
        run_bot_server(ctx.config, ctx.pool)
            .await
            .map_err(Into::into)
    }
}
