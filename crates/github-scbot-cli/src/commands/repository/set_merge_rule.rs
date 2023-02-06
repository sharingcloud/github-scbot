use std::io::Write;

use crate::Result;
use async_trait::async_trait;
use clap::Parser;
use github_scbot_core::types::{
    pulls::GhMergeStrategy, repository::RepositoryPath, rule_branch::RuleBranch,
};
use github_scbot_database::MergeRule;

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
};

/// Set merge rule for a repository
#[derive(Parser)]
pub(crate) struct RepositorySetMergeRuleCommand {
    /// Repository path (e.g. `MyOrganization/my-project`)
    repository_path: RepositoryPath,
    /// Base branch name
    base_branch: RuleBranch,
    /// Head branch name
    head_branch: RuleBranch,
    /// Merge strategy
    strategy: GhMergeStrategy,
}

#[async_trait(?Send)]
impl Command for RepositorySetMergeRuleCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let (owner, name) = self.repository_path.components();
        let repo = CliDbExt::get_existing_repository(ctx.db_adapter.as_mut(), owner, name).await?;

        if self.base_branch == RuleBranch::Wildcard && self.head_branch == RuleBranch::Wildcard {
            // Update default strategy
            ctx.db_adapter
                .repositories_set_default_strategy(owner, name, self.strategy)
                .await?;

            writeln!(
                ctx.writer,
                "Default strategy updated to '{}' for repository '{}'",
                self.strategy, self.repository_path
            )?;
        } else {
            ctx.db_adapter
                .merge_rules_delete(
                    owner,
                    name,
                    self.base_branch.clone(),
                    self.head_branch.clone(),
                )
                .await?;
            ctx.db_adapter
                .merge_rules_create(
                    MergeRule::builder()
                        .repository_id(repo.id())
                        .base_branch(self.base_branch.clone())
                        .head_branch(self.head_branch.clone())
                        .strategy(self.strategy)
                        .build()
                        .unwrap(),
                )
                .await?;

            writeln!(ctx.writer, "Merge rule created/updated with '{}' for repository '{}' and branches '{}' (base) <- '{}' (head)", self.strategy, self.repository_path, self.base_branch, self.head_branch)?;
        }

        Ok(())
    }
}
