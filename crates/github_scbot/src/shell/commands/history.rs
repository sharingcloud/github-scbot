//! History module.

use dialoguer::Confirm;
use github_scbot_conf::Config;
use github_scbot_database::{
    establish_single_connection,
    models::{HistoryWebhookModel, RepositoryModel},
};
use owo_colors::OwoColorize;

use super::errors::Result;

pub(crate) fn list_webhook_events_from_repository(
    config: &Config,
    repository_path: &str,
) -> Result<()> {
    let conn = establish_single_connection(config)?;
    let repo = RepositoryModel::get_from_path(&conn, &repository_path)?;

    let entries = HistoryWebhookModel::list_from_repository_id(&conn, repo.id)?;
    if entries.is_empty() {
        println!("No events for repository {}.", repo.get_path());
    } else {
        for entry in entries {
            println!("{:?}", entry);
        }
    }

    Ok(())
}

pub(crate) fn remove_webhook_events(config: &Config) -> Result<()> {
    let conn = establish_single_connection(config)?;

    let entries = HistoryWebhookModel::list(&conn)?;
    let entries_count = entries.len();
    if entries.is_empty() {
        println!("No events to remove.");
    } else {
        println!("You will remove {} events.", entries_count);

        let prompt = "Do you want to continue?".yellow();
        if Confirm::new().with_prompt(prompt.to_string()).interact()? {
            HistoryWebhookModel::remove_all(&conn)?;
            println!("{} events removed.", entries_count);
        } else {
            println!("Cancelled.");
        }
    }

    Ok(())
}
