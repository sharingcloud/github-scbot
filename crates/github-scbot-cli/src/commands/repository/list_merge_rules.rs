use std::io::Write;

use async_trait::async_trait;
use clap::Parser;
use github_scbot_domain_models::RepositoryPath;

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
    Result,
};

/// List merge rules for a repository
#[derive(Parser)]
pub(crate) struct RepositoryListMergeRulesCommand {
    /// Repository path (e.g. `MyOrganization/my-project`)
    repository_path: RepositoryPath,
}

#[async_trait(?Send)]
impl Command for RepositoryListMergeRulesCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let (owner, name) = self.repository_path.components();

        let repo = CliDbExt::get_existing_repository(ctx.db_service.as_mut(), owner, name).await?;

        let default_strategy = repo.default_strategy;
        let rules = ctx.db_service.merge_rules_list(owner, name).await?;

        writeln!(
            ctx.writer,
            "Merge rules for repository {}:",
            self.repository_path
        )?;
        writeln!(ctx.writer, "- Default: '{}'", default_strategy)?;
        for rule in rules {
            writeln!(
                ctx.writer,
                "- '{}' (base) <- '{}' (head): '{}'",
                rule.base_branch, rule.head_branch, rule.strategy
            )?;
        }

        Ok(())
    }
}
