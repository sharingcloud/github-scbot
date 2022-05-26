use std::io::Write;

use crate::Result;
use async_trait::async_trait;
use clap::Parser;
use github_scbot_tui::run_tui;
use snafu::ResultExt;

use super::{Command, CommandContext};
use crate::errors::UiSnafu;

/// Start TUI
#[derive(Parser)]
pub(crate) struct UiCommand;

#[async_trait(?Send)]
impl Command for UiCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        run_tui(&mut *ctx.db_adapter).await.context(UiSnafu)
    }
}
