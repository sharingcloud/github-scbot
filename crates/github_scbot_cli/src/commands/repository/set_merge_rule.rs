use std::convert::TryFrom;

use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_database::models::{MergeRuleModel, RepositoryModel};
use github_scbot_types::pulls::GhMergeStrategy;
use stable_eyre::eyre::Result;

use crate::commands::{Command, CommandContext};

/// set merge rule for a repository.
#[derive(FromArgs)]
#[argh(subcommand, name = "set-merge-rule")]
pub(crate) struct RepositorySetMergeRuleCommand {
    /// repository path (e.g. `MyOrganization/my-project`).
    #[argh(positional)]
    repository_path: String,
    /// base branch name.
    #[argh(positional)]
    base_branch: String,
    /// head branch name.
    #[argh(positional)]
    head_branch: String,
    /// merge strategy.
    #[argh(positional)]
    strategy: String,
}

#[async_trait(?Send)]
impl Command for RepositorySetMergeRuleCommand {
    async fn execute<'a>(self, ctx: CommandContext<'a>) -> Result<()> {
        let strategy_enum = GhMergeStrategy::try_from(&self.strategy[..])?;
        let mut repo =
            RepositoryModel::get_from_path(ctx.db_adapter.repository(), &self.repository_path)
                .await?;

        if &self.base_branch == "*" && &self.head_branch == "*" {
            // Update default strategy
            repo.set_default_merge_strategy(strategy_enum);
            ctx.db_adapter.repository().save(&mut repo).await?;

            println!(
                "Default strategy updated to '{}' for repository '{}'",
                self.strategy, self.repository_path
            );
        } else {
            MergeRuleModel::builder(&repo, &self.base_branch[..], &self.head_branch[..])
                .strategy(strategy_enum)
                .create_or_update(ctx.db_adapter.merge_rule())
                .await?;
            println!("Merge rule created/updated with '{}' for repository '{}' and branches '{}' (base) <- '{}' (head)", self.strategy, self.repository_path, self.base_branch, self.head_branch);
        }

        Ok(())
    }
}
