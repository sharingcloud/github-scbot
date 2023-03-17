use std::io::Write;

use crate::Result;
use anyhow::anyhow;
use async_trait::async_trait;
use clap::Parser;
use github_scbot_core::types::{repository::RepositoryPath, rule_branch::RuleBranch};

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
};

/// Remove merge rule for a repository
#[derive(Parser)]
pub(crate) struct RepositoryRemoveMergeRuleCommand {
    /// Repository path (e.g. `MyOrganization/my-project`)
    repository_path: RepositoryPath,
    /// Base branch name
    base_branch: RuleBranch,
    /// Head branch name
    head_branch: RuleBranch,
}

#[async_trait(?Send)]
impl Command for RepositoryRemoveMergeRuleCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let (owner, name) = self.repository_path.components();
        let _repo = CliDbExt::get_existing_repository(ctx.db_service.as_mut(), owner, name).await?;

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
                    ctx.writer,
                    "Merge rule for repository '{}' and branches '{}' (base) <- '{}' (head) deleted.",
                    self.repository_path, self.base_branch, self.head_branch
                )?;
            } else {
                writeln!(
                    ctx.writer,
                    "Unknown merge rule for repository '{}' and branches '{}' (base) <- '{}' (head).",
                    self.repository_path, self.base_branch, self.head_branch
                )?;
            }
        }

        Ok(())
    }
}
