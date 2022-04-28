use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_sentry::eyre::Result;
use github_scbot_types::repository::RepositoryPath;

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
};

/// list merge rules for a repository.
#[derive(FromArgs)]
#[argh(subcommand, name = "list-merge-rule")]
pub(crate) struct RepositoryListMergeRulesCommand {
    /// repository path (e.g. `MyOrganization/my-project`).
    #[argh(positional)]
    repository_path: RepositoryPath,
}

#[async_trait(?Send)]
impl Command for RepositoryListMergeRulesCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let (owner, name) = self.repository_path.components();

        let mut repo_db = ctx.db_adapter.repositories();
        let repo = CliDbExt::get_existing_repository(&mut *repo_db, owner, name).await?;

        let default_strategy = repo.default_strategy();
        let rules = ctx.db_adapter.merge_rules().list(owner, name).await?;

        println!("Merge rules for repository {}:", self.repository_path);
        println!("- Default: '{}'", default_strategy.to_string());
        for rule in rules {
            println!(
                "- '{}' (base) <- '{}' (head): '{}'",
                rule.base_branch(),
                rule.head_branch(),
                rule.strategy()
            );
        }

        Ok(())
    }
}
