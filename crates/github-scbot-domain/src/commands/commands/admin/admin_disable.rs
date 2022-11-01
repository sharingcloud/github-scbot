use github_scbot_core::types::issues::GhReactionType;

use async_trait::async_trait;

use crate::{
    commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    status::StatusLogic,
    Result,
};

pub struct AdminDisableCommand;

impl AdminDisableCommand {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait(?Send)]
impl BotCommand for AdminDisableCommand {
    async fn handle(&self, ctx: &CommandContext) -> Result<CommandExecutionResult> {
        let repo_model = ctx
            .db_adapter
            .repositories()
            .get(ctx.repo_owner, ctx.repo_name)
            .await?
            .unwrap();

        if repo_model.manual_interaction() {
            StatusLogic::disable_validation_status(
                ctx.api_adapter,
                ctx.db_adapter,
                ctx.repo_owner,
                ctx.repo_name,
                ctx.pr_number,
            )
            .await?;

            ctx.db_adapter
                .pull_requests()
                .delete(ctx.repo_owner, ctx.repo_name, ctx.pr_number)
                .await?;

            let comment = "Bot disabled on this PR. Bye!";
            Ok(CommandExecutionResult::builder()
                .with_status_update(false)
                .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
                .with_action(ResultAction::PostComment(comment.into()))
                .build())
        } else {
            let comment = "You can not disable the bot on this PR, the repository is not in manual interaction mode.";
            Ok(CommandExecutionResult::builder()
                .denied()
                .with_status_update(false)
                .with_action(ResultAction::AddReaction(GhReactionType::MinusOne))
                .with_action(ResultAction::PostComment(comment.into()))
                .build())
        }
    }
}

#[cfg(test)]
mod tests {
    use futures_util::FutureExt;
    use github_scbot_core::types::pulls::GhPullRequest;
    use github_scbot_database::{MockPullRequestDB, MockRepositoryDB, PullRequest, Repository};
    use mockall::predicate;

    use crate::commands::CommandContextTest;

    use super::*;

    #[actix_rt::test]
    async fn test_no_manual_interaction() -> Result<()> {
        let mut ctx = CommandContextTest::new();

        ctx.db_adapter.expect_repositories().times(1).returning(|| {
            let mut mock = MockRepositoryDB::new();
            mock.expect_get()
                .times(1)
                .with(predicate::eq("owner"), predicate::eq("name"))
                .returning(|_, _| {
                    async { Ok(Some(Repository::builder().build().unwrap())) }.boxed()
                });

            Box::new(mock)
        });

        let result = AdminDisableCommand::new().handle(&ctx.as_context()).await?;
        assert!(!result.should_update_status);
        assert_eq!(result.result_actions, vec![
            ResultAction::AddReaction(GhReactionType::MinusOne),
            ResultAction::PostComment("You can not disable the bot on this PR, the repository is not in manual interaction mode.".into())
        ]);

        Ok(())
    }

    #[actix_rt::test]
    async fn test_manual_interaction() -> Result<()> {
        let mut ctx = CommandContextTest::new();

        ctx.db_adapter.expect_repositories().times(1).returning(|| {
            let mut mock = MockRepositoryDB::new();
            mock.expect_get()
                .times(1)
                .with(predicate::eq("owner"), predicate::eq("name"))
                .returning(|_, _| {
                    async {
                        Ok(Some(
                            Repository::builder()
                                .manual_interaction(true)
                                .build()
                                .unwrap(),
                        ))
                    }
                    .boxed()
                });

            Box::new(mock)
        });

        ctx.api_adapter
            .expect_pulls_get()
            .times(1)
            .with(
                predicate::eq("owner"),
                predicate::eq("name"),
                predicate::eq(1),
            )
            .returning(|_, _, _| Ok(GhPullRequest::default()));
        ctx.api_adapter
            .expect_commit_statuses_update()
            .times(1)
            .returning(|_, _, _, _, _, _| Ok(()));

        ctx.db_adapter
            .expect_pull_requests()
            .times(2)
            .returning(|| {
                let mut mock = MockPullRequestDB::new();
                mock.expect_get().returning(|_, _, _| {
                    async { Ok(Some(PullRequest::builder().build().unwrap())) }.boxed()
                });

                mock.expect_delete()
                    .with(
                        predicate::eq("owner"),
                        predicate::eq("name"),
                        predicate::eq(1),
                    )
                    .returning(|_, _, _| async { Ok(true) }.boxed());

                Box::new(mock)
            });

        let result = AdminDisableCommand::new().handle(&ctx.as_context()).await?;
        assert!(!result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![
                ResultAction::AddReaction(GhReactionType::Eyes),
                ResultAction::PostComment("Bot disabled on this PR. Bye!".into())
            ]
        );

        Ok(())
    }
}
