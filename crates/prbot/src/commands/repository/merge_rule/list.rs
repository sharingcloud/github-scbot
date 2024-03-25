use async_trait::async_trait;
use clap::Parser;
use prbot_models::RepositoryPath;

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
    Result,
};

/// List merge rules for a repository
#[derive(Parser)]
pub(crate) struct ListCommand {
    /// Repository path (e.g. `MyOrganization/my-project`)
    repository_path: RepositoryPath,
}

#[async_trait]
impl Command for ListCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let (owner, name) = self.repository_path.components();

        let repo = CliDbExt::get_existing_repository(ctx.db_service.as_ref(), owner, name).await?;

        let default_strategy = repo.default_strategy;
        let rules = ctx.db_service.merge_rules_list(owner, name).await?;

        writeln!(
            ctx.writer.write().await,
            "Merge rules for repository {}:",
            self.repository_path
        )?;
        writeln!(
            ctx.writer.write().await,
            "- Default: '{}'",
            default_strategy
        )?;
        for rule in rules {
            writeln!(
                ctx.writer.write().await,
                "- '{}' (base) <- '{}' (head): '{}'",
                rule.base_branch,
                rule.head_branch,
                rule.strategy
            )?;
        }

        Ok(())
    }
}
