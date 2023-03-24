use std::fmt::Write;

use async_trait::async_trait;
use github_scbot_ghapi_interface::types::GhReactionType;

use crate::{
    commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    use_cases::reviews::{
        AddReviewersUseCase, AddReviewersUseCaseInterface, FilterReviewersUseCase,
        FilteredReviewers, RemoveReviewersUseCase, RemoveReviewersUseCaseInterface,
    },
    Result,
};

pub struct SetReviewersCommand {
    reviewers: Vec<String>,
    action: Action,
    required: bool,
}

struct AssignReviewersCommand {
    reviewers: Vec<String>,
    required: bool,
}

impl AssignReviewersCommand {
    async fn handle<'a>(&self, ctx: &CommandContext<'a>) -> Result<CommandExecutionResult> {
        let reviewers = AddReviewersUseCase {
            api_service: ctx.api_service,
            db_service: ctx.db_service,
            filter_reviewers: &FilterReviewersUseCase {
                api_service: ctx.api_service,
            },
        }
        .run(&ctx.pr_handle(), &self.reviewers, self.required)
        .await?;

        let comment = self._create_status_message(&reviewers);

        Ok(CommandExecutionResult::builder()
            .with_status_update(!reviewers.allowed.is_empty())
            .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
            .with_action(ResultAction::PostComment(comment))
            .build())
    }

    fn _create_status_message(&self, reviewers: &FilteredReviewers) -> String {
        let mut comment = String::new();
        let allowed_len = reviewers.allowed.len();
        let rejected_len = reviewers.rejected.len();

        let subject = if self.required {
            "required reviewer"
        } else {
            "reviewer"
        };

        match allowed_len {
            0 => (),
            1 => write!(
                comment,
                "**{}** is now a {subject} on this pull request.",
                reviewers.allowed[0]
            )
            .unwrap(),
            _ => write!(
                comment,
                "**{}** are now {subject}s on this pull request.",
                reviewers.allowed.join(", ")
            )
            .unwrap(),
        }

        if allowed_len > 0 && rejected_len > 0 {
            comment.push_str("\n\nBut");
        }

        match rejected_len {
            0 => (),
            1 => write!(
                comment,
                "**{}** has no write permission on this repository and can't be a reviewer.",
                reviewers.rejected[0]
            )
            .unwrap(),
            _ => write!(
                comment,
                "**{}** have no write permission on this repository and can't be reviewers.",
                reviewers.rejected.join(", ")
            )
            .unwrap(),
        }

        comment
    }
}

struct UnassignReviewersCommand {
    reviewers: Vec<String>,
}

impl UnassignReviewersCommand {
    async fn handle<'a>(&self, ctx: &CommandContext<'a>) -> Result<CommandExecutionResult> {
        RemoveReviewersUseCase {
            api_service: ctx.api_service,
            db_service: ctx.db_service,
        }
        .run(&ctx.pr_handle(), &self.reviewers)
        .await?;

        let comment = self._create_status_message();

        Ok(CommandExecutionResult::builder()
            .with_status_update(true)
            .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
            .with_action(ResultAction::PostComment(comment))
            .build())
    }

    fn _create_status_message(&self) -> String {
        if self.reviewers.len() == 1 {
            format!(
                "**{}** is not a reviewer anymore on this pull request.",
                self.reviewers[0]
            )
        } else {
            format!(
                "**{}** are not reviewers anymore on this pull request.",
                self.reviewers.join(", ")
            )
        }
    }
}

enum Action {
    Assign,
    Unassign,
}

impl SetReviewersCommand {
    pub fn new_assign(reviewers: Vec<String>, required: bool) -> Self {
        Self {
            action: Action::Assign,
            reviewers,
            required,
        }
    }

    pub fn new_unassign(reviewers: Vec<String>) -> Self {
        Self {
            action: Action::Unassign,
            reviewers,
            required: false,
        }
    }
}

#[async_trait(?Send)]
impl BotCommand for SetReviewersCommand {
    async fn handle(&self, ctx: &CommandContext) -> Result<CommandExecutionResult> {
        match self.action {
            Action::Assign => {
                AssignReviewersCommand {
                    reviewers: self.reviewers.clone(),
                    required: self.required,
                }
                .handle(ctx)
                .await
            }
            Action::Unassign => {
                UnassignReviewersCommand {
                    reviewers: self.reviewers.clone(),
                }
                .handle(ctx)
                .await
            }
        }
    }
}
