use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_sentry::eyre::{eyre::eyre, Result};
use github_scbot_types::{repository::RepositoryPath, rule_branch::RuleBranch};

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
};

/// remove merge rule for a repository.
#[derive(FromArgs)]
#[argh(subcommand, name = "remove-merge-rule")]
pub(crate) struct RepositoryRemoveMergeRuleCommand {
    /// repository path (e.g. `MyOrganization/my-project`).
    #[argh(positional)]
    repository_path: RepositoryPath,
    /// base branch name.
    #[argh(positional)]
    base_branch: RuleBranch,
    /// head branch name.
    #[argh(positional)]
    head_branch: RuleBranch,
}

#[async_trait(?Send)]
impl Command for RepositoryRemoveMergeRuleCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let (owner, name) = self.repository_path.components();
        let _repo =
            CliDbExt::get_existing_repository(&mut *ctx.db_adapter.repositories(), owner, name)
                .await?;

        if self.base_branch == RuleBranch::Wildcard && self.head_branch == RuleBranch::Wildcard {
            return Err(eyre!("Cannot remove default strategy"));
        } else {
            let found = ctx
                .db_adapter
                .merge_rules()
                .delete(
                    owner,
                    name,
                    self.base_branch.clone(),
                    self.head_branch.clone(),
                )
                .await?;
            if found {
                println!(
                    "Merge rule for repository '{}' and branches '{}' (base) <- '{}' (head) deleted.",
                    self.repository_path, self.base_branch, self.head_branch
                );
            } else {
                eprintln!(
                    "Unknown merge rule for repository '{}' and branches '{}' (base) <- '{}' (head).",
                    self.repository_path, self.base_branch, self.head_branch
                )
            }
        }

        Ok(())
    }
}
