use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_database::models::RepositoryModel;
use stable_eyre::eyre::Result;

use crate::shell::commands::{Command, CommandContext};

/// list merge rules for a repository.
#[derive(FromArgs)]
#[argh(subcommand, name = "list-merge-rule")]
pub(crate) struct RepositoryListMergeRulesCommand {
    /// repository path (e.g. `MyOrganization/my-project`).
    #[argh(positional)]
    repository_path: String,
}

#[async_trait(?Send)]
impl Command for RepositoryListMergeRulesCommand {
    async fn execute<'a>(self, ctx: CommandContext<'a>) -> Result<()> {
        let repo =
            RepositoryModel::get_from_path(ctx.db_adapter.repository(), &self.repository_path)
                .await?;
        let default_strategy = repo.get_default_merge_strategy();
        let rules = ctx
            .db_adapter
            .merge_rule()
            .list_from_repository_id(repo.id)
            .await?;

        println!("Merge rules for repository {}:", self.repository_path);
        println!("- Default: '{}'", default_strategy.to_string());
        for rule in rules {
            println!(
                "- '{}' (base) <- '{}' (head): '{}'",
                rule.base_branch,
                rule.head_branch,
                rule.get_strategy().to_string()
            );
        }

        Ok(())
    }
}
