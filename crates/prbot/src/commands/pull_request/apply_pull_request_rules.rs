use async_trait::async_trait;
use clap::Parser;
use prbot_core::use_cases::pulls::{
    ApplyPullRequestRulesInterface, ResolvePullRequestRulesInterface,
};
use prbot_models::{PullRequestHandle, RepositoryPath};
use shaku::HasComponent;

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
    Result,
};

/// Apply matching pull request rules for a pull request
#[derive(Parser)]
pub(crate) struct PullRequestApplyPullRequestRulesCommand {
    /// Repository path (e.g. 'MyOrganization/my-project')
    repository_path: RepositoryPath,

    /// Pull request number
    number: u64,
}

#[async_trait]
impl Command for PullRequestApplyPullRequestRulesCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let (owner, name) = self.repository_path.components();

        let _pr =
            CliDbExt::get_existing_pull_request(ctx.db_service.as_ref(), owner, name, self.number)
                .await?;

        let upstream_pr = ctx.api_service.pulls_get(owner, name, self.number).await?;

        let resolve_rules: &dyn ResolvePullRequestRulesInterface = ctx.core_module.resolve_ref();
        let rules = resolve_rules
            .run(&ctx.as_core_context(), &self.repository_path, &upstream_pr)
            .await?;

        if rules.is_empty() {
            writeln!(ctx.writer.write().await, "No rule to apply.")?;
            return Ok(());
        } else {
            writeln!(
                ctx.writer.write().await,
                "Will apply rules '{}'.",
                rules
                    .iter()
                    .map(|r| r.name.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            )?;
        }

        let handle = PullRequestHandle::new(self.repository_path.clone(), self.number);
        let apply_rules: &dyn ApplyPullRequestRulesInterface = ctx.core_module.resolve_ref();
        apply_rules
            .run(&ctx.as_core_context(), &handle, rules)
            .await?;

        Ok(())
    }
}
