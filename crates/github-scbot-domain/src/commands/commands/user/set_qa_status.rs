use github_scbot_core::types::{issues::GhReactionType, status::QaStatus};

use async_trait::async_trait;

use crate::{
    commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    Result,
};

pub struct SetQaStatusCommand {
    status: QaStatus,
}

impl SetQaStatusCommand {
    pub fn new(status: QaStatus) -> Self {
        Self { status }
    }

    pub fn new_skip_or_wait(status: bool) -> Self {
        Self {
            status: if status {
                QaStatus::Skipped
            } else {
                QaStatus::Waiting
            },
        }
    }

    pub fn new_pass_or_fail(status: Option<bool>) -> Self {
        Self {
            status: match status {
                None => QaStatus::Waiting,
                Some(true) => QaStatus::Pass,
                Some(false) => QaStatus::Fail,
            },
        }
    }

    fn _create_status_message<'a>(&self, ctx: &CommandContext<'a>) -> String {
        let status = match self.status {
            QaStatus::Fail => "failed",
            QaStatus::Pass => "passed",
            QaStatus::Skipped => "skipped",
            QaStatus::Waiting => "waiting",
        };

        format!(
            "QA status is marked as **{}** by **{}**.",
            status, ctx.comment_author
        )
    }
}

#[async_trait(?Send)]
impl BotCommand for SetQaStatusCommand {
    async fn handle(&self, ctx: &CommandContext) -> Result<CommandExecutionResult> {
        ctx.db_adapter
            .pull_requests()
            .set_qa_status(ctx.repo_owner, ctx.repo_name, ctx.pr_number, self.status)
            .await?;

        let comment = self._create_status_message(ctx);

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
    use github_scbot_core::types::status::QaStatus;
    use github_scbot_database::MockPullRequestDB;
    use github_scbot_database::PullRequest;
    use mockall::predicate;

    use crate::commands::CommandContextTest;

    use super::*;

    #[actix_rt::test]
    async fn test_skip() -> Result<()> {
        let mut ctx = CommandContextTest::new();

        // Skip.
        ctx.db_adapter
            .expect_pull_requests()
            .times(1)
            .returning(|| {
                let mut mock = MockPullRequestDB::new();
                mock.expect_set_qa_status()
                    .with(
                        predicate::eq("owner"),
                        predicate::eq("name"),
                        predicate::eq(1),
                        predicate::eq(QaStatus::Skipped),
                    )
                    .returning(|_, _, _, _| {
                        async { Ok(PullRequest::builder().build().unwrap()) }.boxed()
                    });
                Box::new(mock)
            });

        let cmd = SetQaStatusCommand::new(QaStatus::Skipped);
        let result = cmd.handle(&ctx.as_context()).await?;
        assert!(result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![
                ResultAction::AddReaction(GhReactionType::Eyes),
                ResultAction::PostComment("QA status is marked as **skipped** by **me**.".into())
            ]
        );

        Ok(())
    }

    #[actix_rt::test]
    async fn test_reset() -> Result<()> {
        let mut ctx = CommandContextTest::new();

        // Reset.
        ctx.db_adapter
            .expect_pull_requests()
            .times(1)
            .returning(|| {
                let mut mock = MockPullRequestDB::new();
                mock.expect_set_qa_status()
                    .with(
                        predicate::eq("owner"),
                        predicate::eq("name"),
                        predicate::eq(1),
                        predicate::eq(QaStatus::Waiting),
                    )
                    .returning(|_, _, _, _| {
                        async { Ok(PullRequest::builder().build().unwrap()) }.boxed()
                    });
                Box::new(mock)
            });

        let cmd = SetQaStatusCommand::new(QaStatus::Waiting);
        let result = cmd.handle(&ctx.as_context()).await?;
        assert!(result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![
                ResultAction::AddReaction(GhReactionType::Eyes),
                ResultAction::PostComment("QA status is marked as **waiting** by **me**.".into())
            ]
        );

        Ok(())
    }

    #[actix_rt::test]
    async fn test_approve() -> Result<()> {
        let mut ctx = CommandContextTest::new();

        // Reset.
        ctx.db_adapter
            .expect_pull_requests()
            .times(1)
            .returning(|| {
                let mut mock = MockPullRequestDB::new();
                mock.expect_set_qa_status()
                    .with(
                        predicate::eq("owner"),
                        predicate::eq("name"),
                        predicate::eq(1),
                        predicate::eq(QaStatus::Pass),
                    )
                    .returning(|_, _, _, _| {
                        async { Ok(PullRequest::builder().build().unwrap()) }.boxed()
                    });
                Box::new(mock)
            });

        let cmd = SetQaStatusCommand::new(QaStatus::Pass);
        let result = cmd.handle(&ctx.as_context()).await?;
        assert!(result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![
                ResultAction::AddReaction(GhReactionType::Eyes),
                ResultAction::PostComment("QA status is marked as **passed** by **me**.".into())
            ]
        );

        Ok(())
    }

    #[actix_rt::test]
    async fn test_unapprove() -> Result<()> {
        let mut ctx = CommandContextTest::new();

        // Reset.
        ctx.db_adapter
            .expect_pull_requests()
            .times(1)
            .returning(|| {
                let mut mock = MockPullRequestDB::new();
                mock.expect_set_qa_status()
                    .with(
                        predicate::eq("owner"),
                        predicate::eq("name"),
                        predicate::eq(1),
                        predicate::eq(QaStatus::Fail),
                    )
                    .returning(|_, _, _, _| {
                        async { Ok(PullRequest::builder().build().unwrap()) }.boxed()
                    });
                Box::new(mock)
            });

        let cmd = SetQaStatusCommand::new(QaStatus::Fail);
        let result = cmd.handle(&ctx.as_context()).await?;
        assert!(result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![
                ResultAction::AddReaction(GhReactionType::Eyes),
                ResultAction::PostComment("QA status is marked as **failed** by **me**.".into())
            ]
        );

        Ok(())
    }
}
