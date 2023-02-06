use std::io::Write;

use crate::Result;
use async_trait::async_trait;
use clap::Parser;
use github_scbot_tui::run_tui;

use super::{Command, CommandContext};

/// Start TUI
#[derive(Parser)]
pub(crate) struct UiCommand;

#[async_trait(?Send)]
impl Command for UiCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        run_tui(ctx.db_adapter.as_mut()).await.map_err(Into::into)
    }
}
