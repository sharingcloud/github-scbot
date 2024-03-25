use async_trait::async_trait;
use clap::Parser;
use prbot_server::server::{run_bot_server, AppContext};

use super::{Command, CommandContext};
use crate::Result;

/// Start server
#[derive(Parser)]
pub(crate) struct ServerCommand;

#[async_trait]
impl Command for ServerCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        tokio::task::spawn_local(async move {
            let context = AppContext::new_with_adapters(
                ctx.config,
                ctx.core_module,
                ctx.db_service,
                ctx.api_service,
                ctx.lock_service,
            );

            run_bot_server(context).await.unwrap();
        })
        .await?;

        Ok(())
    }
}
