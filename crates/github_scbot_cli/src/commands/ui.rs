use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_sentry::eyre::Result;
use github_scbot_tui::run_tui;

use super::{Command, CommandContext};

/// start TUI.
#[derive(FromArgs)]
#[argh(subcommand, name = "ui")]
pub(crate) struct UiCommand {}

#[async_trait(?Send)]
impl Command for UiCommand {
    async fn execute(self, mut ctx: CommandContext) -> Result<()> {
        run_tui(&mut *ctx.db_adapter).await.map_err(Into::into)
    }
}
