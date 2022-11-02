use github_scbot_core::types::issues::GhReactionType;
use github_scbot_database::{PullRequest, RequiredReviewer};
use std::fmt::Write;

use async_trait::async_trait;

use crate::{
    commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    Result,
};

pub struct SetRequiredReviewersCommand {
    reviewers: Vec<String>,
    action: Action,
}

struct AssignRequiredReviewersCommand {
    reviewers: Vec<String>,
}

struct FilteredReviewers {
    approved: Vec<String>,
    rejected: Vec<String>,
}

impl FilteredReviewers {
    async fn new<'a>(
        ctx: &CommandContext<'a>,
        pr_model: &PullRequest,
        reviewers: &[String],
    ) -> Result<Self> {
        let mut approved = vec![];
        let mut rejected = vec![];

        for reviewer in reviewers {
            let permission = ctx
                .api_adapter
                .user_permissions_get(ctx.repo_owner, ctx.repo_name, reviewer)
                .await?
                .can_write();

            if permission {
                approved.push(reviewer.clone());
                Self::_create_reviewer(ctx, pr_model, reviewer).await?;
            } else {
                rejected.push(reviewer.clone());
            }
        }

        Ok(Self { approved, rejected })
    }

    fn has_approvals(&self) -> bool {
        !self.approved.is_empty()
    }

    async fn _create_reviewer<'a>(
        ctx: &CommandContext<'a>,
        pr_model: &PullRequest,
        reviewer_username: &str,
    ) -> Result<()> {
        match ctx
            .db_adapter
            .required_reviewers()
            .get(
                ctx.repo_owner,
                ctx.repo_name,
                ctx.pr_number,
                reviewer_username,
            )
            .await?
        {
            Some(_s) => Ok(()),
            None => {
                ctx.db_adapter
                    .required_reviewers()
                    .create(
                        RequiredReviewer::builder()
                            .with_pull_request(pr_model)
                            .username(reviewer_username)
                            .build()
                            .unwrap(),
                    )
                    .await?;
                Ok(())
            }
        }
    }
}

impl AssignRequiredReviewersCommand {
    async fn handle<'a>(&self, ctx: &CommandContext<'a>) -> Result<CommandExecutionResult> {
        let pr_model = self._get_pull_request(ctx).await?;
        let reviewers = self._filter_reviewers(ctx, &pr_model).await?;

        if reviewers.has_approvals() {
            self._set_github_reviewers(ctx, &reviewers.approved).await?;
        }

        let comment = self._create_status_message(&reviewers);

        Ok(CommandExecutionResult::builder()
            .with_status_update(reviewers.has_approvals())
            .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
            .with_action(ResultAction::PostComment(comment))
            .build())
    }

    async fn _filter_reviewers<'a>(
        &self,
        ctx: &CommandContext<'a>,
        pr_model: &PullRequest,
    ) -> Result<FilteredReviewers> {
        FilteredReviewers::new(ctx, pr_model, &self.reviewers).await
    }

    async fn _get_pull_request<'a>(&self, ctx: &CommandContext<'a>) -> Result<PullRequest> {
        Ok(ctx
            .db_adapter
            .pull_requests()
            .get(ctx.repo_owner, ctx.repo_name, ctx.pr_number)
            .await?
            .unwrap())
    }

    async fn _set_github_reviewers<'a>(
        &self,
        ctx: &CommandContext<'a>,
        reviewers: &[String],
    ) -> Result<()> {
        ctx.api_adapter
            .pull_reviewer_requests_add(ctx.repo_owner, ctx.repo_name, ctx.pr_number, reviewers)
            .await?;

        Ok(())
    }

    fn _create_status_message(&self, reviewers: &FilteredReviewers) -> String {
        let mut comment = String::new();
        let approved_len = reviewers.approved.len();
        let rejected_len = reviewers.rejected.len();

        match approved_len {
            0 => (),
            1 => write!(
                comment,
                "**{}** is now a required reviewer on this pull request.",
                reviewers.approved[0]
            )
            .unwrap(),
            _ => write!(
                comment,
                "**{}** are now required reviewers on this pull request.",
                reviewers.approved.join(", ")
            )
            .unwrap(),
        }

        if approved_len > 0 && rejected_len > 0 {
            comment.push_str("\n\nBut");
        }

        match rejected_len {
            0 => (),
            1 => write!(
                comment,
                "**{}** has no write permission on this repository and can't be a required reviewer.",
                reviewers.rejected[0]
            )
            .unwrap(),
            _ => write!(
                comment,
                "**{}** have no write permission on this repository and can't be required reviewers.",
                reviewers.rejected.join(", ")
            )
            .unwrap(),
        }

        comment
    }
}

struct UnassignRequiredReviewersCommand {
    reviewers: Vec<String>,
}

impl UnassignRequiredReviewersCommand {
    async fn handle<'a>(&self, ctx: &CommandContext<'a>) -> Result<CommandExecutionResult> {
        self._remove_reviewer_from_pull_request(ctx).await?;

        for reviewer in &self.reviewers {
            self._remove_required_reviewer(ctx, reviewer).await?;
        }

        let comment = self._create_status_message();

        Ok(CommandExecutionResult::builder()
            .with_status_update(true)
            .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
            .with_action(ResultAction::PostComment(comment))
            .build())
    }

    async fn _remove_reviewer_from_pull_request<'a>(&self, ctx: &CommandContext<'a>) -> Result<()> {
        ctx.api_adapter
            .pull_reviewer_requests_remove(
                ctx.repo_owner,
                ctx.repo_name,
                ctx.pr_number,
                &self.reviewers,
            )
            .await?;

        Ok(())
    }

    async fn _remove_required_reviewer<'a>(
        &self,
        ctx: &CommandContext<'a>,
        reviewer_username: &str,
    ) -> Result<()> {
        ctx.db_adapter
            .required_reviewers()
            .delete(
                ctx.repo_owner,
                ctx.repo_name,
                ctx.pr_number,
                reviewer_username,
            )
            .await?;

        Ok(())
    }

    fn _create_status_message(&self) -> String {
        if self.reviewers.len() == 1 {
            format!(
                "**{}** is not a required reviewer anymore on this pull request.",
                self.reviewers[0]
            )
        } else {
            format!(
                "**{}** are not required reviewers anymore on this pull request.",
                self.reviewers.join(", ")
            )
        }
    }
}

enum Action {
    Assign,
    Unassign,
}

impl SetRequiredReviewersCommand {
    pub fn new_assign(reviewers: Vec<String>) -> Self {
        Self {
            action: Action::Assign,
            reviewers,
        }
    }

    pub fn new_unassign(reviewers: Vec<String>) -> Self {
        Self {
            action: Action::Unassign,
            reviewers,
        }
    }
}

#[async_trait(?Send)]
impl BotCommand for SetRequiredReviewersCommand {
    async fn handle(&self, ctx: &CommandContext) -> Result<CommandExecutionResult> {
        match self.action {
            Action::Assign => {
                AssignRequiredReviewersCommand {
                    reviewers: self.reviewers.clone(),
                }
                .handle(ctx)
                .await
            }
            Action::Unassign => {
                UnassignRequiredReviewersCommand {
                    reviewers: self.reviewers.clone(),
                }
                .handle(ctx)
                .await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use futures_util::FutureExt;
    use github_scbot_core::types::common::GhUserPermission;
    use github_scbot_database::{MockPullRequestDB, MockRequiredReviewerDB};

    use crate::commands::CommandContextTest;

    use super::*;

    #[actix_rt::test]
    async fn test_assign() -> Result<()> {
        let mut ctx = CommandContextTest::new();
        ctx.db_adapter
            .expect_pull_requests()
            .times(1)
            .returning(|| {
                let mut mock = MockPullRequestDB::new();
                mock.expect_get().times(1).returning(|_, _, _| {
                    async { Ok(Some(PullRequest::builder().build().unwrap())) }.boxed()
                });

                Box::new(mock)
            });
        ctx.db_adapter
            .expect_required_reviewers()
            .times(4)
            .returning(|| {
                let mut mock = MockRequiredReviewerDB::new();
                mock.expect_get()
                    .returning(|_, _, _, _| async { Ok(None) }.boxed());
                mock.expect_create().returning(|_| {
                    async { Ok(RequiredReviewer::builder().build().unwrap()) }.boxed()
                });

                Box::new(mock)
            });
        ctx.api_adapter
            .expect_user_permissions_get()
            .times(2)
            .returning(|_, _, _| Ok(GhUserPermission::Write));
        ctx.api_adapter
            .expect_pull_reviewer_requests_add()
            .times(1)
            .returning(|_, _, _, _| Ok(()));

        let cmd =
            SetRequiredReviewersCommand::new_assign(vec![String::from("one"), String::from("two")]);
        let result = cmd.handle(&ctx.as_context()).await?;
        assert!(result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![
                ResultAction::AddReaction(GhReactionType::Eyes),
                ResultAction::PostComment(
                    "**one, two** are now required reviewers on this pull request.".into()
                )
            ]
        );

        Ok(())
    }

    #[actix_rt::test]
    async fn test_unassign() -> Result<()> {
        let mut ctx = CommandContextTest::new();
        ctx.db_adapter
            .expect_required_reviewers()
            .times(2)
            .returning(|| {
                let mut mock = MockRequiredReviewerDB::new();
                mock.expect_delete()
                    .times(1)
                    .returning(|_, _, _, _| async { Ok(true) }.boxed());

                Box::new(mock)
            });

        ctx.api_adapter
            .expect_pull_reviewer_requests_remove()
            .times(1)
            .returning(|_, _, _, _| Ok(()));

        let cmd = SetRequiredReviewersCommand::new_unassign(vec![
            String::from("one"),
            String::from("two"),
        ]);
        let result = cmd.handle(&ctx.as_context()).await?;
        assert!(result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![
                ResultAction::AddReaction(GhReactionType::Eyes),
                ResultAction::PostComment(
                    "**one, two** are not required reviewers anymore on this pull request.".into()
                )
            ]
        );

        Ok(())
    }
}