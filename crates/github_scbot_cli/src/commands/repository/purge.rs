use argh::FromArgs;
use async_trait::async_trait;
use dialoguer::Confirm;
use github_scbot_database::models::RepositoryModel;
use owo_colors::OwoColorize;
use stable_eyre::eyre::Result;

use crate::commands::{Command, CommandContext};

/// purge closed pull requests for a repository.
#[derive(FromArgs)]
#[argh(subcommand, name = "purge")]
pub(crate) struct RepositoryPurgeCommand {
    /// repository path (e.g. `MyOrganization/my-project`).
    #[argh(positional)]
    repository_path: String,
}

#[async_trait(?Send)]
impl Command for RepositoryPurgeCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let repo =
            RepositoryModel::get_from_path(ctx.db_adapter.repository(), &self.repository_path)
                .await?;

        let prs_to_purge = ctx
            .db_adapter
            .pull_request()
            .list_closed_pulls_from_repository(repo.id)
            .await?;
        if prs_to_purge.is_empty() {
            println!(
                "No closed pull request to remove for repository '{}'",
                self.repository_path
            );
        } else {
            println!(
                "You will remove:\n{}",
                prs_to_purge
                    .iter()
                    .map(|p| format!("- #{} - {}", p.get_number(), p.name))
                    .collect::<Vec<_>>()
                    .join("\n")
            );

            let prompt = "Do you want to continue?".yellow();
            if Confirm::new().with_prompt(prompt.to_string()).interact()? {
                ctx.db_adapter
                    .pull_request()
                    .remove_closed_pulls_from_repository(repo.id)
                    .await?;
                println!("{} pull requests removed.", prs_to_purge.len());
            } else {
                println!("Cancelled.");
            }
        }

        Ok(())
    }
}
