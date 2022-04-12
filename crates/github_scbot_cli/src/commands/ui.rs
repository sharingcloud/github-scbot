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
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        todo!()
        // run_tui(ctx.db_adapter.as_ref()).await.map_err(Into::into)
    }
}
