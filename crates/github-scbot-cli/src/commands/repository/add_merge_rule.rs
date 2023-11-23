use std::io::Write;

use async_trait::async_trait;
use clap::Parser;
use github_scbot_domain::use_cases::repositories::{
    AddMergeRuleUseCase, AddMergeRuleUseCaseInterface,
};
use github_scbot_domain_models::{MergeStrategy, RepositoryPath, RuleBranch};

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
    Result,
};

/// Add merge rule for a repository
#[derive(Parser)]
pub(crate) struct RepositoryAddMergeRuleCommand {
    /// Repository path (e.g. `MyOrganization/my-project`)
    repository_path: RepositoryPath,
    /// Base branch name
    base_branch: RuleBranch,
    /// Head branch name
    head_branch: RuleBranch,
    /// Merge strategy
    strategy: MergeStrategy,
}

#[async_trait(?Send)]
impl Command for RepositoryAddMergeRuleCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let (owner, name) = self.repository_path.components();
        let repo = CliDbExt::get_existing_repository(ctx.db_service.as_ref(), owner, name).await?;

        AddMergeRuleUseCase {
            db_service: &*ctx.db_service,
        }
        .run(
            &repo,
            self.base_branch.clone(),
            self.head_branch.clone(),
            self.strategy,
        )
        .await?;

        if self.base_branch == RuleBranch::Wildcard && self.head_branch == RuleBranch::Wildcard {
            writeln!(
                ctx.writer,
                "Default strategy updated to '{}' for repository '{}'",
                self.strategy, self.repository_path
            )?;
        } else {
            writeln!(ctx.writer, "Merge rule created/updated with '{}' for repository '{}' and branches '{}' (base) <- '{}' (head)", self.strategy, self.repository_path, self.base_branch, self.head_branch)?;
        }

        Ok(())
    }
}
