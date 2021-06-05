//! History module.

use dialoguer::Confirm;
use github_scbot_database::models::{IDatabaseAdapter, RepositoryModel};
use owo_colors::OwoColorize;

use super::errors::Result;

pub(crate) async fn list_webhook_events_from_repository(
    db_adapter: &dyn IDatabaseAdapter,
    repository_path: &str,
) -> Result<()> {
    let repo = RepositoryModel::get_from_path(db_adapter.repository(), repository_path).await?;
    let entries = db_adapter
        .history_webhook()
        .list_from_repository_id(repo.id)
        .await?;
    if entries.is_empty() {
        println!("No events for repository {}.", repo.get_path());
    } else {
        for entry in entries {
            entry.show();
        }
    }

    Ok(())
}

pub(crate) async fn remove_webhook_events(
    db_adapter: &dyn IDatabaseAdapter,
    no_input: bool,
) -> Result<()> {
    let entries = db_adapter.history_webhook().list().await?;
    let entries_count = entries.len();
    if entries.is_empty() {
        println!("No events to remove.");
    } else {
        println!("You will remove {} events.", entries_count);

        let prompt = "Do you want to continue?".yellow();
        if no_input || Confirm::new().with_prompt(prompt.to_string()).interact()? {
            db_adapter.history_webhook().remove_all().await?;
            println!("{} events removed.", entries_count);
        } else {
            println!("Cancelled.");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use github_scbot_database::models::DummyDatabaseAdapter;

    use super::*;

    #[actix_rt::test]
    async fn test_list_webhook_events_from_repository() -> Result<()> {
        let db_adapter = DummyDatabaseAdapter::new();
        list_webhook_events_from_repository(&db_adapter, "test/repo").await
    }

    #[actix_rt::test]
    async fn test_remove_webhook_events() -> Result<()> {
        let db_adapter = DummyDatabaseAdapter::new();
        remove_webhook_events(&db_adapter, true).await
    }
}
