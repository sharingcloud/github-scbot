use github_scbot_core::types::{issues::GhReactionType, labels::StepLabel, pulls::GhMergeStrategy};

use async_trait::async_trait;
use tracing::error;

use crate::{
    commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    pulls::PullRequestLogic,
    status::{PullRequestStatus, StatusLogic},
    Result,
};

pub struct MergeCommand {
    strategy: Option<GhMergeStrategy>,
}

impl MergeCommand {
    pub fn new(strategy: Option<GhMergeStrategy>) -> Self {
        Self { strategy }
    }

    pub fn new_default_strategy() -> Self {
        Self { strategy: None }
    }
}

#[async_trait(?Send)]
impl BotCommand for MergeCommand {
    async fn handle(&self, ctx: &CommandContext) -> Result<CommandExecutionResult> {
        let pr_status = PullRequestStatus::from_database(
            ctx.api_adapter,
            ctx.db_adapter,
            ctx.repo_owner,
            ctx.repo_name,
            ctx.pr_number,
            ctx.upstream_pr,
        )
        .await?;
        let step = StatusLogic::determine_automatic_step(&pr_status);
        let commit_title = PullRequestLogic::get_merge_commit_title(ctx.upstream_pr);
        let mut actions = vec![];

        // Use step to determine merge possibility
        if step == StepLabel::AwaitingMerge {
            let strategy = self.strategy.unwrap_or(pr_status.merge_strategy);
            if let Err(e) = ctx
                .api_adapter
                .pulls_merge(
                    ctx.repo_owner,
                    ctx.repo_name,
                    ctx.pr_number,
                    &commit_title,
                    "",
                    strategy,
                )
                .await
            {
                error!(
                    owner = %ctx.repo_owner,
                    name = %ctx.repo_name,
                    pr_number = ctx.pr_number,
                    error = %e,
                    message = "Error while merging pull request"
                );

                actions.push(ResultAction::AddReaction(GhReactionType::MinusOne));
                actions.push(ResultAction::PostComment(
                    "Error while merging this pull request.".to_string(),
                ));
            } else {
                actions.push(ResultAction::AddReaction(GhReactionType::PlusOne));
                actions.push(ResultAction::PostComment(format!(
                    "Pull request successfully merged by **{}**! (strategy: **{}**)",
                    ctx.comment_author, strategy
                )));
            }
        } else {
            actions.push(ResultAction::AddReaction(GhReactionType::MinusOne));
            actions.push(ResultAction::PostComment(
                "Pull request is not ready to merge.".into(),
            ));
        }

        Ok(CommandExecutionResult::builder()
            .with_status_update(true)
            .with_actions(actions)
            .build())
    }
}

#[cfg(test)]
mod tests {
    use futures_util::FutureExt;
    use github_scbot_core::types::status::QaStatus;
    use github_scbot_database::MockMergeRuleDB;
    use github_scbot_database::MockPullRequestDB;
    use github_scbot_database::MockRepositoryDB;
    use github_scbot_database::MockRequiredReviewerDB;
    use github_scbot_database::PullRequest;
    use github_scbot_database::Repository;
    use github_scbot_ghapi::ApiError;
    use mockall::predicate;

    use crate::commands::CommandContextTest;

    use super::*;

    fn setup_context(ctx: &mut CommandContextTest) {
        ctx.upstream_pr.title = "Title".into();
        ctx.upstream_pr.mergeable = Some(true);

        ctx.api_adapter
            .expect_pull_reviews_list()
            .times(1)
            .returning(|_, _, _| Ok(vec![]));

        ctx.db_adapter
            .expect_pull_requests()
            .times(1)
            .returning(|| {
                let mut mock = MockPullRequestDB::new();
                mock.expect_get().returning(|_, _, _| {
                    async {
                        Ok(Some(
                            PullRequest::builder()
                                .checks_enabled(false)
                                .qa_status(QaStatus::Skipped)
                                .build()
                                .unwrap(),
                        ))
                    }
                    .boxed()
                });

                Box::new(mock)
            });
        ctx.db_adapter.expect_repositories().times(1).returning(|| {
            let mut mock = MockRepositoryDB::new();
            mock.expect_get().returning(|_, _| {
                async { Ok(Some(Repository::builder().build().unwrap())) }.boxed()
            });

            Box::new(mock)
        });
        ctx.db_adapter
            .expect_required_reviewers()
            .times(1)
            .returning(|| {
                let mut mock = MockRequiredReviewerDB::new();
                mock.expect_list()
                    .returning(|_, _, _| async { Ok(vec![]) }.boxed());

                Box::new(mock)
            });
        ctx.db_adapter.expect_merge_rules().times(1).returning(|| {
            let mut mock = MockMergeRuleDB::new();
            mock.expect_get()
                .returning(|_, _, _, _| async { Ok(None) }.boxed());

            Box::new(mock)
        });
    }

    #[actix_rt::test]
    async fn test_fail_wip() -> Result<()> {
        let mut ctx = CommandContextTest::new();
        setup_context(&mut ctx);
        ctx.upstream_pr.draft = true;

        let cmd = MergeCommand::new_default_strategy();
        let result = cmd.handle(&ctx.as_context()).await?;

        assert!(result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![
                ResultAction::AddReaction(GhReactionType::MinusOne),
                ResultAction::PostComment("Pull request is not ready to merge.".into())
            ]
        );

        Ok(())
    }

    #[actix_rt::test]
    async fn test_fail_github() -> Result<()> {
        let mut ctx = CommandContextTest::new();
        setup_context(&mut ctx);

        // Setup an error while merging
        ctx.api_adapter
            .expect_pulls_merge()
            .times(1)
            .returning(|_, _, _, _, _, _| {
                Err(ApiError::MergeError {
                    pr_number: 1,
                    repository_path: "owner/name".into(),
                })
            });

        let cmd = MergeCommand::new_default_strategy();
        let result = cmd.handle(&ctx.as_context()).await?;
        assert!(result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![
                ResultAction::AddReaction(GhReactionType::MinusOne),
                ResultAction::PostComment("Error while merging this pull request.".into())
            ]
        );

        Ok(())
    }

    #[actix_rt::test]
    async fn test_success() -> Result<()> {
        let mut ctx = CommandContextTest::new();
        setup_context(&mut ctx);

        ctx.api_adapter
            .expect_pulls_merge()
            .times(1)
            .with(
                predicate::eq("owner"),
                predicate::eq("name"),
                predicate::eq(1),
                predicate::eq("Title (#0)"),
                predicate::eq(""),
                predicate::eq(GhMergeStrategy::Merge),
            )
            .returning(|_, _, _, _, _, _| Ok(()));

        let cmd = MergeCommand::new_default_strategy();
        let result = cmd.handle(&ctx.as_context()).await?;

        assert!(result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![
                ResultAction::AddReaction(GhReactionType::PlusOne),
                ResultAction::PostComment(
                    "Pull request successfully merged by **me**! (strategy: **merge**)".into()
                )
            ]
        );

        Ok(())
    }

    #[actix_rt::test]
    async fn test_success_override() -> Result<()> {
        let mut ctx = CommandContextTest::new();
        setup_context(&mut ctx);

        ctx.api_adapter
            .expect_pulls_merge()
            .times(1)
            .with(
                predicate::eq("owner"),
                predicate::eq("name"),
                predicate::eq(1),
                predicate::eq("Title (#0)"),
                predicate::eq(""),
                predicate::eq(GhMergeStrategy::Squash),
            )
            .returning(|_, _, _, _, _, _| Ok(()));

        let cmd = MergeCommand::new(Some(GhMergeStrategy::Squash));
        let result = cmd.handle(&ctx.as_context()).await?;

        assert!(result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![
                ResultAction::AddReaction(GhReactionType::PlusOne),
                ResultAction::PostComment(
                    "Pull request successfully merged by **me**! (strategy: **squash**)".into()
                )
            ]
        );

        Ok(())
    }
}
