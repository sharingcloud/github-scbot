use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_database2::MergeRule;
use github_scbot_sentry::eyre::Result;
use github_scbot_types::{
    pulls::GhMergeStrategy, repository::RepositoryPath, rule_branch::RuleBranch,
};

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
};

/// set merge rule for a repository.
#[derive(FromArgs)]
#[argh(subcommand, name = "set-merge-rule")]
pub(crate) struct RepositorySetMergeRuleCommand {
    /// repository path (e.g. `MyOrganization/my-project`).
    #[argh(positional)]
    repository_path: RepositoryPath,
    /// base branch name.
    #[argh(positional)]
    base_branch: RuleBranch,
    /// head branch name.
    #[argh(positional)]
    head_branch: RuleBranch,
    /// merge strategy.
    #[argh(positional)]
    strategy: GhMergeStrategy,
}

#[async_trait(?Send)]
impl Command for RepositorySetMergeRuleCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let (owner, name) = self.repository_path.components();
        let mut pr_repo = ctx.db_adapter.repositories();
        let repo = CliDbExt::get_existing_repository(&mut *pr_repo, owner, name).await?;

        if self.base_branch == RuleBranch::Wildcard && self.head_branch == RuleBranch::Wildcard {
            // Update default strategy
            pr_repo
                .set_default_strategy(owner, name, self.strategy)
                .await?;

            println!(
                "Default strategy updated to '{}' for repository '{}'",
                self.strategy, self.repository_path
            );
        } else {
            let mut mr_repo = ctx.db_adapter.merge_rules();
            mr_repo
                .delete(
                    owner,
                    name,
                    self.base_branch.clone(),
                    self.head_branch.clone(),
                )
                .await?;
            mr_repo
                .create(
                    MergeRule::builder()
                        .repository_id(repo.id())
                        .base_branch(self.base_branch.clone())
                        .head_branch(self.head_branch.clone())
                        .strategy(self.strategy)
                        .build()?,
                )
                .await?;

            println!("Merge rule created/updated with '{}' for repository '{}' and branches '{}' (base) <- '{}' (head)", self.strategy, self.repository_path, self.base_branch, self.head_branch);
        }

        Ok(())
    }
}
