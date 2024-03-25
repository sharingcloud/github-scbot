use async_trait::async_trait;
use clap::Parser;
use prbot_models::RepositoryPath;

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
    Result,
};

/// Remove pull request rule for a repository
#[derive(Parser)]
pub(crate) struct ListCommand {
    /// Repository path (e.g. `MyOrganization/my-project`)
    repository_path: RepositoryPath,
}

#[async_trait]
impl Command for ListCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let (owner, name) = self.repository_path.components();
        CliDbExt::get_existing_repository(ctx.db_service.as_ref(), owner, name).await?;

        let rules = ctx.db_service.pull_request_rules_list(owner, name).await?;
        if rules.is_empty() {
            writeln!(
                ctx.writer.write().await,
                "No pull request rule found for repository '{}'",
                self.repository_path
            )?;
        } else {
            writeln!(
                ctx.writer.write().await,
                "Pull request rules for repository '{}':",
                self.repository_path
            )?;

            for rule in rules {
                writeln!(ctx.writer.write().await, "  - {:?}", rule)?;
            }
        }

        Ok(())
    }
}
