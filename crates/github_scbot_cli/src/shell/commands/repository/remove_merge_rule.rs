use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_database::models::{MergeRuleModel, RepositoryModel};
use stable_eyre::eyre::{eyre, Result};

use crate::shell::commands::{Command, CommandContext};

/// remove merge rule for a repository.
#[derive(FromArgs)]
#[argh(subcommand, name = "remove-merge-rule")]
pub(crate) struct RepositoryRemoveMergeRuleCommand {
    /// repository path (e.g. `MyOrganization/my-project`).
    #[argh(positional)]
    repository_path: String,
    /// base branch name.
    #[argh(positional)]
    base_branch: String,
    /// head branch name.
    #[argh(positional)]
    head_branch: String,
}

#[async_trait(?Send)]
impl Command for RepositoryRemoveMergeRuleCommand {
    async fn execute<'a>(self, ctx: CommandContext<'a>) -> Result<()> {
        let repo =
            RepositoryModel::get_from_path(ctx.db_adapter.repository(), &self.repository_path)
                .await?;

        if &self.base_branch == "*" && &self.head_branch == "*" {
            return Err(eyre!("Cannot remove default strategy"));
        } else {
            // Try to get rule
            let rule = MergeRuleModel::get_from_branches(
                ctx.db_adapter.merge_rule(),
                &repo,
                &self.base_branch[..],
                &self.head_branch[..],
            )
            .await?;
            ctx.db_adapter.merge_rule().remove(rule).await?;
            println!(
                "Merge rule for repository '{}' and branches '{}' (base) <- '{}' (head) deleted.",
                self.repository_path, self.base_branch, self.head_branch
            );
        }

        Ok(())
    }
}
