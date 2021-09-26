use argh::FromArgs;
use async_trait::async_trait;
use stable_eyre::eyre::Result;

use super::{Command, CommandContext};

mod list_webhook_events;
mod remove_webhook_events;

use list_webhook_events::HistoryListWebhookEventsCommand;
use remove_webhook_events::HistoryRemoveWebhookEventsCommand;

/// history related commands.
#[derive(FromArgs)]
#[argh(subcommand, name = "history")]
pub(crate) struct HistoryCommand {
    #[argh(subcommand)]
    inner: HistorySubCommand,
}

#[async_trait(?Send)]
impl Command for HistoryCommand {
    async fn execute<'a>(self, ctx: CommandContext<'a>) -> Result<()> {
        self.inner.execute(ctx).await
    }
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum HistorySubCommand {
    ListWebhookEvents(HistoryListWebhookEventsCommand),
    RemoveWebhookEvents(HistoryRemoveWebhookEventsCommand),
}

#[async_trait(?Send)]
impl Command for HistorySubCommand {
    async fn execute<'a>(self, ctx: CommandContext<'a>) -> Result<()> {
        match self {
            Self::ListWebhookEvents(sub) => sub.execute(ctx).await,
            Self::RemoveWebhookEvents(sub) => sub.execute(ctx).await,
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use github_scbot_database::models::DummyDatabaseAdapter;

//     use super::*;

//     #[actix_rt::test]
//     async fn test_list_webhook_events_from_repository() -> Result<()> {
//         let db_adapter = DummyDatabaseAdapter::new();
//         list_webhook_events_from_repository(&db_adapter, "test/repo").await
//     }

//     #[actix_rt::test]
//     async fn test_remove_webhook_events() -> Result<()> {
//         let db_adapter = DummyDatabaseAdapter::new();
//         remove_webhook_events(&db_adapter, true).await
//     }
// }
