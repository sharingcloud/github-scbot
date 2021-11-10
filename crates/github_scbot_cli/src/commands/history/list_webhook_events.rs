use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_database::models::RepositoryModel;
use github_scbot_sentry::eyre::Result;

use crate::commands::{Command, CommandContext};

/// list webhook events for repository.
#[derive(FromArgs)]
#[argh(subcommand, name = "list-webhook-events")]
pub(crate) struct HistoryListWebhookEventsCommand {
    /// repository path (e.g. 'MyOrganization/my-project').
    #[argh(positional)]
    repository_path: String,
}

#[async_trait(?Send)]
impl Command for HistoryListWebhookEventsCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let repo =
            RepositoryModel::get_from_path(ctx.db_adapter.repository(), &self.repository_path)
                .await?;
        let entries = ctx
            .db_adapter
            .history_webhook()
            .list_from_repository_id(repo.id())
            .await?;
        if entries.is_empty() {
            println!("No events for repository {}.", repo.path());
        } else {
            for entry in entries {
                entry.show();
            }
        }

        Ok(())
    }
}
