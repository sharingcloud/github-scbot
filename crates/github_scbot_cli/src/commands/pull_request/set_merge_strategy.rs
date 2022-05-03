use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_sentry::eyre::Result;
use github_scbot_types::{pulls::GhMergeStrategy, repository::RepositoryPath};

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
};

/// list known pull request for a repository.
#[derive(FromArgs)]
#[argh(subcommand, name = "set-merge-strategy")]
pub(crate) struct PullRequestSetMergeStrategyCommand {
    /// repository path (e.g. 'MyOrganization/my-project')
    #[argh(positional)]
    repository_path: RepositoryPath,

    /// pull request number.
    #[argh(positional)]
    number: u64,

    /// merge strategy.
    #[argh(positional)]
    strategy: Option<GhMergeStrategy>,
}

#[async_trait(?Send)]
impl Command for PullRequestSetMergeStrategyCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let (owner, name) = self.repository_path.components();

        let mut pr_db = ctx.db_adapter.pull_requests();
        let _pr =
            CliDbExt::get_existing_pull_request(&mut *pr_db, owner, name, self.number).await?;
        pr_db
            .set_strategy_override(owner, name, self.number, self.strategy)
            .await?;

        if let Some(s) = self.strategy {
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
