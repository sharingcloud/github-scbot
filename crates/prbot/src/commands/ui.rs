use async_trait::async_trait;
use clap::Parser;
use prbot_logging::temporarily_disable_logging;
use prbot_tui::run_tui;

use super::{Command, CommandContext};
use crate::Result;

/// Start TUI
#[derive(Parser)]
pub(crate) struct UiCommand;

#[async_trait]
impl Command for UiCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let _guard = temporarily_disable_logging();
        run_tui(ctx.db_service.as_ref()).await.map_err(Into::into)
    }
}
