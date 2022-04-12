use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_sentry::eyre::Result;
use github_scbot_server::server::{run_bot_server, AppContext};

use super::{Command, CommandContext};

/// start server.
#[derive(FromArgs)]
#[argh(subcommand, name = "server")]
pub(crate) struct ServerCommand {}

#[async_trait(?Send)]
impl Command for ServerCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        todo!()
        // let context = AppContext::new_with_adapters(
        //     ctx.config,
        //     ctx.db_adapter,
        //     ctx.api_adapter,
        //     ctx.redis_adapter,
        // );

        // run_bot_server(context).await.map_err(Into::into)
    }
}
