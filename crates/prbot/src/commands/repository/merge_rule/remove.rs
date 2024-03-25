use anyhow::anyhow;
use async_trait::async_trait;
use clap::Parser;
use prbot_models::{RepositoryPath, RuleBranch};

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
    Result,
};

/// Remove merge rule for a repository
#[derive(Parser)]
pub(crate) struct RemoveCommand {
    /// Repository path (e.g. `MyOrganization/my-project`)
    repository_path: RepositoryPath,
    /// Base branch name
    base_branch: RuleBranch,
    /// Head branch name
    head_branch: RuleBranch,
}

#[async_trait]
impl Command for RemoveCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let (owner, name) = self.repository_path.components();
        let _repo = CliDbExt::get_existing_repository(ctx.db_service.as_ref(), owner, name).await?;

        if self.base_branch == RuleBranch::Wildcard && self.head_branch == RuleBranch::Wildcard {
            return Err(anyhow!("Cannot remove default strategy"));
        } else {
            let found = ctx
                .db_service
                .merge_rules_delete(
                    owner,
                    name,
                    self.base_branch.clone(),
                    self.head_branch.clone(),
                )
                .await?;
            if found {
                writeln!(
                    ctx.writer.write().await,
                    "Merge rule for repository '{}' and branches '{}' (base) <- '{}' (head) deleted.",
                    self.repository_path, self.base_branch, self.head_branch
                )?;
            } else {
                writeln!(
                    ctx.writer.write().await,
                    "Unknown merge rule for repository '{}' and branches '{}' (base) <- '{}' (head).",
                    self.repository_path, self.base_branch, self.head_branch
                )?;
            }
        }

        Ok(())
    }
}
