use std::io::Write;

use async_trait::async_trait;
use clap::Parser;
use github_scbot_logging::temporarily_disable_logging;
use github_scbot_tui::run_tui;

use super::{Command, CommandContext};
use crate::Result;

/// Start TUI
#[derive(Parser)]
pub(crate) struct UiCommand;

#[async_trait(?Send)]
impl Command for UiCommand {
    async fn execute<W: Write>(self, ctx: CommandContext<W>) -> Result<()> {
        let _guard = temporarily_disable_logging();
        run_tui(ctx.db_service.as_ref()).await.map_err(Into::into)
    }
}
