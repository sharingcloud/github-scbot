use async_trait::async_trait;
use clap::Parser;
use prbot_core::use_cases::repositories::AddMergeRuleInterface;
use prbot_models::{MergeStrategy, RepositoryPath, RuleBranch};
use shaku::HasComponent;

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
    Result,
};

/// Add merge rule for a repository
#[derive(Parser)]
pub(crate) struct AddCommand {
    /// Repository path (e.g. `MyOrganization/my-project`)
    repository_path: RepositoryPath,
    /// Base branch name
    base_branch: RuleBranch,
    /// Head branch name
    head_branch: RuleBranch,
    /// Merge strategy
    strategy: MergeStrategy,
}

#[async_trait]
impl Command for AddCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let (owner, name) = self.repository_path.components();
        let repo = CliDbExt::get_existing_repository(ctx.db_service.as_ref(), owner, name).await?;

        let uc: &dyn AddMergeRuleInterface = ctx.core_module.resolve_ref();
        uc.run(
            &ctx.as_core_context(),
            &repo,
            self.base_branch.clone(),
            self.head_branch.clone(),
            self.strategy,
        )
        .await?;

        if self.base_branch == RuleBranch::Wildcard && self.head_branch == RuleBranch::Wildcard {
            writeln!(
                ctx.writer.write().await,
                "Default strategy updated to '{}' for repository '{}'",
                self.strategy,
                self.repository_path
            )?;
        } else {
            writeln!(ctx.writer.write().await, "Merge rule created/updated with '{}' for repository '{}' and branches '{}' (base) <- '{}' (head)", self.strategy, self.repository_path, self.base_branch, self.head_branch)?;
        }

        Ok(())
    }
}
