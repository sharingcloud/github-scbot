use argh::FromArgs;
use async_trait::async_trait;
use dialoguer::Confirm;
use github_scbot_sentry::eyre::Result;
use owo_colors::OwoColorize;

use crate::commands::{Command, CommandContext};

/// remove all webhook events.
#[derive(FromArgs)]
#[argh(subcommand, name = "remove-webhook-events")]
pub(crate) struct HistoryRemoveWebhookEventsCommand {}

#[async_trait(?Send)]
impl Command for HistoryRemoveWebhookEventsCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let entries = ctx.db_adapter.history_webhook().list().await?;
        let entries_count = entries.len();
        if entries.is_empty() {
            println!("No events to remove.");
        } else {
            println!("You will remove {} events.", entries_count);

            let prompt = "Do you want to continue?".yellow();
            if ctx.no_input || Confirm::new().with_prompt(prompt.to_string()).interact()? {
                ctx.db_adapter.history_webhook().remove_all().await?;
                println!("{} events removed.", entries_count);
            } else {
                println!("Cancelled.");
            }
        }

        Ok(())
    }
}
