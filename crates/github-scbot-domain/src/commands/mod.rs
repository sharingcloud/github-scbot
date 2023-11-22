//! Commands module.

mod command;
#[allow(clippy::module_inception)]
pub mod commands;
mod parser;

use async_trait::async_trait;
#[cfg(test)]
pub(crate) use commands::tests::CommandContextTest;
pub use commands::{BotCommand, CommandContext};
use github_scbot_database_interface::DbService;
use github_scbot_ghapi_interface::{
    comments::CommentApi,
    types::{GhReactionType, GhUserPermission},
};
pub use parser::CommandParser;

pub use self::command::{
    AdminCommand, Command, CommandExecutionResult, CommandHandlingStatus, CommandResult,
    ResultAction, UserCommand,
};
use self::commands::AdminAddMergeRuleCommand;
use crate::{
    commands::commands::{
        AdminDisableCommand, AdminHelpCommand, AdminResetSummaryCommand,
        AdminSetDefaultAutomergeCommand, AdminSetDefaultChecksStatusCommand,
        AdminSetDefaultMergeStrategyCommand, AdminSetDefaultPrTitleRegexCommand,
        AdminSetDefaultQaStatusCommand, AdminSetDefaultReviewersCommand,
        AdminSetPrReviewersCommand, AdminSyncCommand, GifCommand, HelpCommand, IsAdminCommand,
        LockCommand, MergeCommand, PingCommand, SetAutomergeCommand, SetChecksStatusCommand,
        SetLabelsCommand, SetMergeStrategyCommand, SetQaStatusCommand, SetReviewersCommand,
    },
    use_cases::{
        auth::{CheckIsAdminUseCase, CheckWriteRightUseCase},
        status::UpdatePullRequestStatusUseCaseInterface,
    },
    Result,
};

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait(?Send)]
pub trait CommandExecutorInterface {
    async fn execute_commands<'a>(
        &self,
        ctx: &CommandContext<'a>,
        commands: Vec<CommandResult<Command>>,
    ) -> Result<CommandExecutionResult>;

    async fn process_command_result<'a>(
        &self,
        ctx: &CommandContext<'a>,
        command_result: &CommandExecutionResult,
    ) -> Result<()>;
}

/// Command executor.
pub struct CommandExecutor<'a> {
    pub db_service: &'a dyn DbService,
    pub update_pull_request_status: &'a dyn UpdatePullRequestStatusUseCaseInterface,
}

#[async_trait(?Send)]
impl<'a> CommandExecutorInterface for CommandExecutor<'a> {
    /// Execute multiple commands.
    #[tracing::instrument(
        skip_all,
        fields(
            repo_owner = %ctx.repo_owner,
            repo_name = %ctx.repo_name,
            pr_number = ctx.pr_number,
            comment_author = %ctx.comment_author,
            commands = ?commands
        ),
        ret
    )]
    async fn execute_commands<'b>(
        &self,
        ctx: &CommandContext<'b>,
        commands: Vec<CommandResult<Command>>,
    ) -> Result<CommandExecutionResult> {
        let mut status = vec![];

        for command in commands {
            match command {
                Ok(command) => {
                    status.push(self.execute_command(ctx, command).await?);
                }
                Err(e) => {
                    // Handle error
                    status.push(
                        CommandExecutionResult::builder()
                            .denied()
                            .with_action(ResultAction::AddReaction(GhReactionType::MinusOne))
                            .with_action(ResultAction::PostComment(format!("{}", e)))
                            .build(),
                    )
                }
            }
        }

        // Merge and handle command result
        let command_result = self.merge_command_results(status);
        self.process_command_result(ctx, &command_result).await?;

        Ok(command_result)
    }

    /// Process command result.
    async fn process_command_result<'b>(
        &self,
        ctx: &CommandContext<'b>,
        command_result: &CommandExecutionResult,
    ) -> Result<()> {
        if command_result.should_update_status {
            // Make sure the upstream is up to date
            let upstream_pr = ctx
                .api_service
                .pulls_get(ctx.repo_owner, ctx.repo_name, ctx.pr_number)
                .await?;

            self.update_pull_request_status
                .run(&ctx.pr_handle(), &upstream_pr)
                .await?;
        }

        for action in &command_result.result_actions {
            match action {
                ResultAction::AddReaction(reaction) => {
                    CommentApi::add_reaction_to_comment(
                        ctx.api_service,
                        ctx.repo_owner,
                        ctx.repo_name,
                        ctx.comment_id,
                        *reaction,
                    )
                    .await?;
                }
                ResultAction::PostComment(comment) => {
                    CommentApi::post_comment(
                        ctx.api_service,
                        ctx.repo_owner,
                        ctx.repo_name,
                        ctx.pr_number,
                        comment,
                    )
                    .await?;
                }
            }
        }

        Ok(())
    }
}

impl<'a> CommandExecutor<'a> {
    /// Merge command results.
    pub fn merge_command_results(
        &self,
        results: Vec<CommandExecutionResult>,
    ) -> CommandExecutionResult {
        let mut handling_status = CommandHandlingStatus::Ignored;
        let mut result_actions = vec![];
        let mut should_update_status = false;

        for result in results {
            use CommandHandlingStatus::{Denied, Handled, Ignored};

            handling_status = match (handling_status, result.handling_status) {
                (Ignored | Denied, Denied) => Denied,
                (_, Handled) | (Handled, _) => Handled,
                (previous, Ignored) => previous,
            };

            should_update_status = match (should_update_status, result.should_update_status) {
                (_, true) | (true, _) => true,
                (false, false) => false,
            };

            result_actions.extend(result.result_actions);
        }

        // Merge actions
        let mut merged_actions = vec![];
        let mut comments = vec![];
        for action in result_actions {
            // If action already present, ignores
            if merged_actions.contains(&action) {
                continue;
            }

            if let ResultAction::PostComment(comment) = action {
                comments.push(comment);
            } else {
                merged_actions.push(action);
            }
        }

        // Create only one comment action
        if !comments.is_empty() {
            merged_actions.push(ResultAction::PostComment(comments.join("\n\n---\n\n")));
        }

        CommandExecutionResult {
            handling_status,
            result_actions: merged_actions,
            should_update_status,
        }
    }

    /// Execute command.
    #[allow(clippy::too_many_lines)]
    #[tracing::instrument(
        skip_all,
        fields(
            repo_owner = %ctx.repo_owner,
            repo_name = %ctx.repo_name,
            pr_number = ctx.pr_number,
            comment_author = %ctx.comment_author,
            command = ?command
        ),
        ret
    )]
    pub async fn execute_command(
        &self,
        ctx: &CommandContext<'_>,
        command: Command,
    ) -> Result<CommandExecutionResult> {
        let mut command_result: CommandExecutionResult;

        let permission = ctx
            .api_service
            .user_permissions_get(ctx.repo_owner, ctx.repo_name, ctx.comment_author)
            .await?;

        if self
            .validate_user_rights_on_command(ctx.comment_author, permission, &command)
            .await?
        {
            command_result = match &command {
                Command::User(cmd) => self._execute_user_command(ctx, cmd).await?,
                Command::Admin(cmd) => self._execute_admin_command(ctx, cmd).await?,
            };

            for action in &mut command_result.result_actions {
                if let ResultAction::PostComment(comment) = action {
                    // Include command recap before comment
                    *comment = format!("> {}\n\n{}", command.to_bot_string(ctx.config), comment);
                }
            }
        } else {
            command_result = CommandExecutionResult::builder()
                .denied()
                .with_action(ResultAction::AddReaction(GhReactionType::MinusOne))
                .build();
        }

        Ok(command_result)
    }

    async fn _execute_user_command(
        &self,
        ctx: &CommandContext<'_>,
        cmd: &UserCommand,
    ) -> Result<CommandExecutionResult> {
        match cmd {
            UserCommand::Automerge(s) => SetAutomergeCommand::new(*s).handle(ctx).await,
            UserCommand::SkipQaStatus(s) => {
                SetQaStatusCommand::new_skip_or_wait(*s).handle(ctx).await
            }
            UserCommand::SkipChecksStatus(s) => {
                SetChecksStatusCommand::new_skip_or_wait(*s)
                    .handle(ctx)
                    .await
            }
            UserCommand::QaStatus(s) => SetQaStatusCommand::new_pass_or_fail(*s).handle(ctx).await,
            UserCommand::Lock(s, reason) => LockCommand::new(*s, reason.clone()).handle(ctx).await,
            UserCommand::Ping => PingCommand::new().handle(ctx).await,
            UserCommand::Merge(strategy) => MergeCommand::new(*strategy).handle(ctx).await,
            UserCommand::AssignReviewers(reviewers) => {
                SetReviewersCommand::new_assign(reviewers.clone(), false)
                    .handle(ctx)
                    .await
            }
            UserCommand::AssignRequiredReviewers(reviewers) => {
                SetReviewersCommand::new_assign(reviewers.clone(), true)
                    .handle(ctx)
                    .await
            }
            UserCommand::SetMergeStrategy(strategy) => {
                SetMergeStrategyCommand::new(*strategy).handle(ctx).await
            }
            UserCommand::UnsetMergeStrategy => {
                SetMergeStrategyCommand::new_unset().handle(ctx).await
            }
            UserCommand::SetLabels(labels) => {
                SetLabelsCommand::new_added(labels.clone())
                    .handle(ctx)
                    .await
            }
            UserCommand::UnsetLabels(labels) => {
                SetLabelsCommand::new_removed(labels.clone())
                    .handle(ctx)
                    .await
            }
            UserCommand::UnassignReviewers(reviewers) => {
                SetReviewersCommand::new_unassign(reviewers.clone())
                    .handle(ctx)
                    .await
            }
            UserCommand::Gif(terms) => GifCommand::new(terms.clone()).handle(ctx).await,
            UserCommand::Help => HelpCommand::new().handle(ctx).await,
            UserCommand::IsAdmin => IsAdminCommand::new().handle(ctx).await,
        }
    }

    async fn _execute_admin_command(
        &self,
        ctx: &CommandContext<'_>,
        cmd: &AdminCommand,
    ) -> Result<CommandExecutionResult> {
        match cmd {
            AdminCommand::Help => AdminHelpCommand::new().handle(ctx).await,
            AdminCommand::Enable => Ok(CommandExecutionResult::builder().ignored().build()),
            AdminCommand::Disable => AdminDisableCommand::new().handle(ctx).await,
            AdminCommand::Synchronize => AdminSyncCommand::new().handle(ctx).await,
            AdminCommand::ResetSummary => AdminResetSummaryCommand::new().handle(ctx).await,
            AdminCommand::AddMergeRule(base, head, strategy) => {
                AdminAddMergeRuleCommand::new(base.clone(), head.clone(), *strategy)
                    .handle(ctx)
                    .await
            }
            AdminCommand::SetDefaultNeededReviewers(count) => {
                AdminSetDefaultReviewersCommand::new(*count)
                    .handle(ctx)
                    .await
            }
            AdminCommand::SetDefaultMergeStrategy(strategy) => {
                AdminSetDefaultMergeStrategyCommand::new(*strategy)
                    .handle(ctx)
                    .await
            }
            AdminCommand::SetDefaultPrTitleRegex(rgx) => {
                AdminSetDefaultPrTitleRegexCommand::new(rgx.clone())
                    .handle(ctx)
                    .await
            }
            AdminCommand::SetDefaultQaStatus(status) => {
                AdminSetDefaultQaStatusCommand::new(*status)
                    .handle(ctx)
                    .await
            }
            AdminCommand::SetDefaultChecksStatus(status) => {
                AdminSetDefaultChecksStatusCommand::new(*status)
                    .handle(ctx)
                    .await
            }
            AdminCommand::SetDefaultAutomerge(value) => {
                AdminSetDefaultAutomergeCommand::new(*value)
                    .handle(ctx)
                    .await
            }
            AdminCommand::SetNeededReviewers(count) => {
                AdminSetPrReviewersCommand::new(*count).handle(ctx).await
            }
        }
    }

    /// Validate user rights on command.
    pub async fn validate_user_rights_on_command(
        &self,
        username: &str,
        user_permission: GhUserPermission,
        command: &Command,
    ) -> Result<bool> {
        match command {
            Command::User(cmd) => match cmd {
                UserCommand::Ping | UserCommand::Help | UserCommand::Gif(_) => Ok(true),
                _ => {
                    CheckWriteRightUseCase {
                        db_service: self.db_service,
                    }
                    .run(username, user_permission)
                    .await
                }
            },
            Command::Admin(_) => {
                CheckIsAdminUseCase {
                    db_service: self.db_service,
                }
                .run(username)
                .await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_database_memory::MemoryDb;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::use_cases::status::MockUpdatePullRequestStatusUseCaseInterface;

    #[test]
    fn test_merge_command_results() {
        let db_service = MemoryDb::new();
        let update_pull_request_status = MockUpdatePullRequestStatusUseCaseInterface::new();
        let executor = CommandExecutor {
            db_service: &db_service,
            update_pull_request_status: &update_pull_request_status,
        };

        let results = vec![
            CommandExecutionResult::builder()
                .denied()
                .with_action(ResultAction::AddReaction(GhReactionType::MinusOne))
                .build(),
            CommandExecutionResult::builder()
                .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
                .with_action(ResultAction::PostComment("Comment 1".into()))
                .build(),
            CommandExecutionResult::builder().ignored().build(),
            CommandExecutionResult::builder()
                .with_status_update(true)
                .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
                .with_action(ResultAction::PostComment("Comment 2".into()))
                .build(),
        ];

        let merged = executor.merge_command_results(results);
        assert_eq!(
            merged,
            CommandExecutionResult {
                handling_status: CommandHandlingStatus::Handled,
                result_actions: vec![
                    ResultAction::AddReaction(GhReactionType::MinusOne),
                    ResultAction::AddReaction(GhReactionType::Eyes),
                    ResultAction::PostComment("Comment 1\n\n---\n\nComment 2".into())
                ],
                should_update_status: true
            }
        );
    }

    #[test]
    fn test_merge_command_results_ignored() {
        let db_service = MemoryDb::new();
        let update_pull_request_status = MockUpdatePullRequestStatusUseCaseInterface::new();
        let executor = CommandExecutor {
            db_service: &db_service,
            update_pull_request_status: &update_pull_request_status,
        };

        let results = vec![
            CommandExecutionResult::builder().ignored().build(),
            CommandExecutionResult::builder().ignored().build(),
        ];

        let merged = executor.merge_command_results(results);
        assert_eq!(
            merged,
            CommandExecutionResult {
                handling_status: CommandHandlingStatus::Ignored,
                result_actions: vec![],
                should_update_status: false
            }
        );
    }
}
