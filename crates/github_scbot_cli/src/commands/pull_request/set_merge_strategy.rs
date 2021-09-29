use std::convert::TryFrom;

use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_types::pulls::GhMergeStrategy;
use stable_eyre::eyre::Result;

use crate::commands::{Command, CommandContext};

/// list known pull request for a repository.
#[derive(FromArgs)]
#[argh(subcommand, name = "set-merge-strategy")]
pub(crate) struct PullRequestSetMergeStrategyCommand {
    /// repository path (e.g. 'MyOrganization/my-project')
    #[argh(positional)]
    repository_path: String,

    /// pull request number.
    #[argh(positional)]
    number: u64,

    /// merge strategy.
    #[argh(positional)]
    strategy: Option<String>,
}

#[async_trait(?Send)]
impl Command for PullRequestSetMergeStrategyCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let strategy_enum = if let Some(s) = self.strategy {
            Some(GhMergeStrategy::try_from(&s[..])?)
        } else {
            None
        };

        let (mut pr, _repo) = ctx
            .db_adapter
            .pull_request()
            .get_from_repository_path_and_number(&self.repository_path, self.number)
            .await?;

        pr.set_strategy_override(strategy_enum);
        ctx.db_adapter.pull_request().save(&mut pr).await?;

        if let Some(s) = strategy_enum {
            println!(
                "Setting {:?} as a merge strategy override for pull request #{} on repository {}",
                s, self.number, self.repository_path
            );
        } else {
            println!(
                "Removing merge strategy override for pull request #{} on repository {}",
                self.number, self.repository_path
            );
        }

        Ok(())
    }
}
