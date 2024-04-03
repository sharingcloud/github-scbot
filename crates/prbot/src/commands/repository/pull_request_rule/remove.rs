use async_trait::async_trait;
use clap::Parser;
use prbot_core::use_cases::pulls::RemovePullRequestRuleInterface;
use prbot_models::RepositoryPath;
use shaku::HasComponent;

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
    Result,
};

/// Remove pull request rule for a repository
#[derive(Parser)]
pub(crate) struct RemoveCommand {
    /// Repository path (e.g. `MyOrganization/my-project`)
    repository_path: RepositoryPath,
    /// Name
    name: String,
}

#[async_trait]
impl Command for RemoveCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let (owner, name) = self.repository_path.components();
        let repo = CliDbExt::get_existing_repository(ctx.db_service.as_ref(), owner, name).await?;

        let uc: &dyn RemovePullRequestRuleInterface = ctx.core_module.resolve_ref();
        uc.run(&ctx.as_core_context(), &repo, &self.name).await?;

        writeln!(
            ctx.writer.write().await,
            "Pull request rule '{}' removed from repository '{}'",
            self.name,
            self.repository_path
        )?;

        Ok(())
    }
}
