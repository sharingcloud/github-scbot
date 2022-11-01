use github_scbot_core::types::issues::GhReactionType;

use async_trait::async_trait;

use crate::{
    commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    Result,
};

pub struct AdminSetDefaultPrTitleRegexCommand {
    regex: String,
}

impl AdminSetDefaultPrTitleRegexCommand {
    pub fn new(regex: String) -> Self {
        Self { regex }
    }
}

#[async_trait(?Send)]
impl BotCommand for AdminSetDefaultPrTitleRegexCommand {
    async fn handle(&self, ctx: &CommandContext) -> Result<CommandExecutionResult> {
        ctx.db_adapter
            .repositories()
            .set_pr_title_validation_regex(ctx.repo_owner, ctx.repo_name, &self.regex)
            .await?;

        let comment = if self.regex.is_empty() {
            "Pull request title regex **unset** for this repository.".into()
        } else {
            format!(
                "Pull request title regex set to **{}** for this repository.",
                self.regex
            )
        };
        Ok(CommandExecutionResult::builder()
            .with_status_update(true)
            .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
            .with_action(ResultAction::PostComment(comment))
            .build())
    }
}

#[cfg(test)]
mod tests {
    use futures_util::FutureExt;
    use github_scbot_database::{MockRepositoryDB, Repository};
    use mockall::predicate;

    use crate::commands::CommandContextTest;

    use super::*;

    #[actix_rt::test]
    async fn test_empty() -> Result<()> {
        let mut ctx = CommandContextTest::new();
        ctx.db_adapter.expect_repositories().returning(|| {
            let mut mock = MockRepositoryDB::new();
            mock.expect_set_pr_title_validation_regex()
                .with(
                    predicate::eq("owner"),
                    predicate::eq("name"),
                    predicate::eq(""),
                )
                .returning(|_, _, _| async { Ok(Repository::builder().build().unwrap()) }.boxed());

            Box::new(mock)
        });

        let result = AdminSetDefaultPrTitleRegexCommand::new("".into())
            .handle(&ctx.as_context())
            .await?;
        assert!(result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![
                ResultAction::AddReaction(GhReactionType::Eyes),
                ResultAction::PostComment(
                    "Pull request title regex **unset** for this repository.".into()
                )
            ]
        );

        Ok(())
    }

    #[actix_rt::test]
    async fn test_non_empty() -> Result<()> {
        let mut ctx = CommandContextTest::new();
        ctx.db_adapter.expect_repositories().returning(|| {
            let mut mock = MockRepositoryDB::new();
            mock.expect_set_pr_title_validation_regex()
                .with(
                    predicate::eq("owner"),
                    predicate::eq("name"),
                    predicate::eq("[a-z]"),
                )
                .returning(|_, _, _| async { Ok(Repository::builder().build().unwrap()) }.boxed());

            Box::new(mock)
        });

        let result = AdminSetDefaultPrTitleRegexCommand::new("[a-z]".into())
            .handle(&ctx.as_context())
            .await?;
        assert!(result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![
                ResultAction::AddReaction(GhReactionType::Eyes),
                ResultAction::PostComment(
                    "Pull request title regex set to **[a-z]** for this repository.".into()
                )
            ]
        );

        Ok(())
    }
}
