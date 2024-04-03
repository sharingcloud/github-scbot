use async_trait::async_trait;
use clap::Parser;
use prbot_core::use_cases::pulls::AddPullRequestRuleInterface;
use prbot_models::{RepositoryPath, RuleAction, RuleCondition};
use shaku::HasComponent;

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
    Result,
};

/// Add pull request rule for a repository
#[derive(Parser)]
pub(crate) struct AddCommand {
    /// Repository path (e.g. `MyOrganization/my-project`)
    repository_path: RepositoryPath,
    /// Name
    name: String,
    /// Conditions
    #[arg(short, long)]
    conditions: Vec<RuleCondition>,
    /// Actions
    #[arg(short, long)]
    actions: Vec<RuleAction>,
}

#[async_trait]
impl Command for AddCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let (owner, name) = self.repository_path.components();
        let repo = CliDbExt::get_existing_repository(ctx.db_service.as_ref(), owner, name).await?;

        let uc: &dyn AddPullRequestRuleInterface = ctx.core_module.resolve_ref();
        uc.run(
            &ctx.as_core_context(),
            &repo,
            self.name.clone(),
            self.conditions,
            self.actions,
        )
        .await?;

        writeln!(
            ctx.writer.write().await,
            "Pull request rule '{}' added to repository '{}'",
            self.name,
            self.repository_path
        )?;

        Ok(())
    }
}
