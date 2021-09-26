use argh::FromArgs;
use async_trait::async_trait;
use stable_eyre::eyre::Result;

use crate::commands::{Command, CommandContext};

/// list known repositories.
#[derive(FromArgs)]
#[argh(subcommand, name = "list")]
pub(crate) struct RepositoryListCommand {}

#[async_trait(?Send)]
impl Command for RepositoryListCommand {
    async fn execute<'a>(self, ctx: CommandContext<'a>) -> Result<()> {
        let repos = ctx.db_adapter.repository().list().await?;
        if repos.is_empty() {
            println!("No repository known.");
        } else {
            for repo in repos {
                println!("- {}/{}", repo.owner, repo.name);
            }
        }

        Ok(())
    }
}
