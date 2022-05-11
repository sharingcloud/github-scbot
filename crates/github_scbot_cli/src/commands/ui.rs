use std::io::Write;

use crate::Result;
use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_tui::run_tui;
use snafu::ResultExt;

use super::{Command, CommandContext};
use crate::errors::UiSnafu;

/// start TUI.
#[derive(FromArgs)]
#[argh(subcommand, name = "ui")]
pub(crate) struct UiCommand {}

#[async_trait(?Send)]
impl Command for UiCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        run_tui(&mut *ctx.db_adapter).await.context(UiSnafu)
    }
}
