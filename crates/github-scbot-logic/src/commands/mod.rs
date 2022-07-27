//! Commands module.

mod command;
mod handlers;
mod parser;

use github_scbot_core::config::Config;
use github_scbot_core::types::{
    common::GhUserPermission, issues::GhReactionType, pulls::GhPullRequest,
};
use github_scbot_database2::DbService;
use github_scbot_ghapi::{adapter::ApiService, comments::CommentApi};
use github_scbot_redis::RedisService;
pub use handlers::handle_qa_command;
pub use parser::CommandParser;

pub use self::command::{AdminCommand, Command, CommandResult, UserCommand};
use super::{errors::Result, status::StatusLogic};
use crate::{
    auth::AuthLogic,
    commands::command::{CommandExecutionResult, CommandHandlingStatus, ResultAction},
};

/// Command executor.
pub struct CommandExecutor;

impl CommandExecutor {
    /// Execute multiple commands.
    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(
        skip_all,
        fields(
            repo_owner = %repo_owner,
            repo_name = %repo_name,
            pr_number = pr_number,
            comment_author = %comment_author,
            commands = ?commands
        ),
        ret
    )]
    pub async fn execute_commands(
        config: &Config,
        api_adapter: &dyn ApiService,
        db_adapter: &dyn DbService,
        redis_adapter: &dyn RedisService,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        upstream_pr: &GhPullRequest,
        comment_id: u64,
        comment_author: &str,
        commands: Vec<CommandResult<Command>>,
    ) -> Result<CommandExecutionResult> {
        let mut status = vec![];

        for command in commands {
            match command {
                Ok(command) => {
                    status.push(
                        Self::execute_command(
                            config,
                            api_adapter,
                            db_adapter,
                            redis_adapter,
                            repo_owner,
                            repo_name,
                            pr_number,
                            upstream_pr,
                            comment_author,
                            command,
                        )
                        .await?,
                    );
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
        let command_result = Self::merge_command_results(status);
        Self::process_command_result(
            api_adapter,
            db_adapter,
            redis_adapter,
            repo_owner,
            repo_name,
            pr_number,
            comment_id,
            &command_result,
        )
        .await?;

        Ok(command_result)
    }

    /// Process command result.
    #[allow(clippy::too_many_arguments)]
    pub async fn process_command_result(
        api_adapter: &dyn ApiService,
        db_adapter: &dyn DbService,
        redis_adapter: &dyn RedisService,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        comment_id: u64,
        command_result: &CommandExecutionResult,
    ) -> Result<()> {
        if command_result.should_update_status {
            // Make sure the upstream is up to date
            let upstream_pr = api_adapter
                .pulls_get(repo_owner, repo_name, pr_number)
                .await?;

            StatusLogic::update_pull_request_status(
                api_adapter,
                db_adapter,
                redis_adapter,
                repo_owner,
                repo_name,
                pr_number,
                &upstream_pr,
            )
            .await?;
        }

        for action in &command_result.result_actions {
            match action {
                ResultAction::AddReaction(reaction) => {
                    CommentApi::add_reaction_to_comment(
                        api_adapter,
                        repo_owner,
                        repo_name,
                        comment_id,
                        *reaction,
                    )
                    .await?;
                }
                ResultAction::PostComment(comment) => {
                    CommentApi::post_comment(
                        api_adapter,
                        repo_owner,
                        repo_name,
                        pr_number,
                        comment,
                    )
                    .await?;
                }
            }
        }

        Ok(())
    }

    /// Merge command results.
    pub fn merge_command_results(results: Vec<CommandExecutionResult>) -> CommandExecutionResult {
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
    #[allow(clippy::too_many_lines, clippy::too_many_arguments)]
    #[tracing::instrument(
        skip_all,
        fields(
            repo_owner = %repo_owner,
            repo_name = %repo_name,
            pr_number = pr_number,
            comment_author = %comment_author,
            command = ?command
        ),
        ret
    )]
    pub async fn execute_command(
        config: &Config,
        api_adapter: &dyn ApiService,
        db_adapter: &dyn DbService,
        redis_adapter: &dyn RedisService,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        upstream_pr: &GhPullRequest,
        comment_author: &str,
        command: Command,
    ) -> Result<CommandExecutionResult> {
        let mut command_result: CommandExecutionResult;

        let permission = api_adapter
            .user_permissions_get(repo_owner, repo_name, comment_author)
            .await?;

        if Self::validate_user_rights_on_command(db_adapter, comment_author, permission, &command)
            .await?
        {
            command_result = match &command {
                Command::User(cmd) => match cmd {
                    UserCommand::Automerge(s) => {
                        handlers::handle_auto_merge_command(
                            db_adapter,
                            repo_owner,
                            repo_name,
                            pr_number,
                            comment_author,
                            *s,
                        )
                        .await?
                    }
                    UserCommand::SkipQaStatus(s) => {
                        handlers::handle_skip_qa_command(
                            db_adapter,
                            repo_owner,
                            repo_name,
                            pr_number,
                            comment_author,
                            *s,
                        )
                        .await?
                    }
                    UserCommand::SkipChecksStatus(s) => {
                        handlers::handle_skip_checks_command(
                            db_adapter,
                            repo_owner,
                            repo_name,
                            pr_number,
                            comment_author,
                            *s,
                        )
                        .await?
                    }
                    UserCommand::QaStatus(s) => {
                        handlers::handle_qa_command(
                            db_adapter,
                            repo_owner,
                            repo_name,
                            pr_number,
                            comment_author,
                            *s,
                        )
                        .await?
                    }
                    UserCommand::Lock(s, reason) => {
                        handlers::handle_lock_command(
                            db_adapter,
                            repo_owner,
                            repo_name,
                            pr_number,
                            comment_author,
                            *s,
                            reason.clone(),
                        )
                        .await?
                    }
                    UserCommand::Ping => handlers::handle_ping_command(comment_author)?,
                    UserCommand::Merge(strategy) => {
                        handlers::handle_merge_command(
                            api_adapter,
                            db_adapter,
                            repo_owner,
                            repo_name,
                            pr_number,
                            upstream_pr,
                            comment_author,
                            *strategy,
                        )
                        .await?
                    }
                    UserCommand::AssignRequiredReviewers(reviewers) => {
                        handlers::handle_assign_required_reviewers_command(
                            api_adapter,
                            db_adapter,
                            repo_owner,
                            repo_name,
                            pr_number,
                            reviewers.clone(),
                        )
                        .await?
                    }
                    UserCommand::SetMergeStrategy(strategy) => {
                        handlers::handle_set_merge_strategy(
                            db_adapter, repo_owner, repo_name, pr_number, *strategy,
                        )
                        .await?
                    }
                    UserCommand::UnsetMergeStrategy => {
                        handlers::handle_unset_merge_strategy(
                            db_adapter, repo_owner, repo_name, pr_number,
                        )
                        .await?
                    }
                    UserCommand::SetLabels(labels) => {
                        handlers::handle_set_labels(
                            api_adapter,
                            repo_owner,
                            repo_name,
                            pr_number,
                            labels,
                        )
                        .await?
                    }
                    UserCommand::UnsetLabels(labels) => {
                        handlers::handle_unset_labels(
                            api_adapter,
                            repo_owner,
                            repo_name,
                            pr_number,
                            labels,
                        )
                        .await?
                    }
                    UserCommand::UnassignRequiredReviewers(reviewers) => {
                        handlers::handle_unassign_required_reviewers_command(
                            api_adapter,
                            db_adapter,
                            repo_owner,
                            repo_name,
                            pr_number,
                            reviewers.clone(),
                        )
                        .await?
                    }
                    UserCommand::Gif(terms) => {
                        handlers::handle_gif_command(config, api_adapter, terms).await?
                    }
                    UserCommand::Help => handlers::handle_help_command(config, comment_author)?,
                    UserCommand::IsAdmin => {
                        handlers::handle_is_admin_command(db_adapter, comment_author).await?
                    }
                },
                Command::Admin(cmd) => match cmd {
                    AdminCommand::Help => {
                        handlers::handle_admin_help_command(config, comment_author)?
                    }
                    AdminCommand::Enable => CommandExecutionResult::builder().ignored().build(),
                    AdminCommand::Disable => {
                        handlers::handle_admin_disable_command(
                            api_adapter,
                            db_adapter,
                            repo_owner,
                            repo_name,
                            pr_number,
                        )
                        .await?
                    }
                    AdminCommand::Synchronize => {
                        handlers::handle_admin_sync_command(
                            config, db_adapter, repo_owner, repo_name, pr_number,
                        )
                        .await?
                    }
                    AdminCommand::ResetSummary => {
                        handlers::handle_admin_reset_summary_command(
                            api_adapter,
                            db_adapter,
                            redis_adapter,
                            repo_owner,
                            repo_name,
                            pr_number,
                            upstream_pr,
                        )
                        .await?
                    }
                    AdminCommand::SetDefaultNeededReviewers(count) => {
                        handlers::handle_set_default_needed_reviewers_command(
                            db_adapter, repo_owner, repo_name, *count,
                        )
                        .await?
                    }
                    AdminCommand::SetDefaultMergeStrategy(strategy) => {
                        handlers::handle_set_default_merge_strategy_command(
                            db_adapter, repo_owner, repo_name, *strategy,
                        )
                        .await?
                    }
                    AdminCommand::SetDefaultPRTitleRegex(rgx) => {
                        handlers::handle_set_default_pr_title_regex_command(
                            db_adapter,
                            repo_owner,
                            repo_name,
                            rgx.clone(),
                        )
                        .await?
                    }
                    AdminCommand::SetDefaultQAStatus(status) => {
                        handlers::handle_set_default_qa_status_command(
                            db_adapter, repo_owner, repo_name, *status,
                        )
                        .await?
                    }
                    AdminCommand::SetDefaultChecksStatus(status) => {
                        handlers::handle_set_default_checks_status_command(
                            db_adapter, repo_owner, repo_name, *status,
                        )
                        .await?
                    }
                    AdminCommand::SetDefaultAutomerge(value) => {
                        handlers::handle_set_default_automerge_command(
                            db_adapter, repo_owner, repo_name, *value,
                        )
                        .await?
                    }
                    AdminCommand::SetNeededReviewers(count) => {
                        handlers::handle_set_needed_reviewers_command(
                            db_adapter, repo_owner, repo_name, pr_number, *count,
                        )
                        .await?
                    }
                },
            };

            for action in &mut command_result.result_actions {
                if let ResultAction::PostComment(comment) = action {
                    // Include command recap before comment
                    *comment = format!("> {}\n\n{}", command.to_bot_string(config), comment);
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

    /// Validate user rights on command.
    pub async fn validate_user_rights_on_command(
        db_adapter: &dyn DbService,
        username: &str,
        user_permission: GhUserPermission,
        command: &Command,
    ) -> Result<bool> {
        let known_admins = AuthLogic::list_known_admin_usernames(db_adapter).await?;

        match command {
            Command::User(cmd) => match cmd {
                UserCommand::Ping | UserCommand::Help | UserCommand::Gif(_) => Ok(true),
                _ => Ok(AuthLogic::has_write_right(
                    username,
                    user_permission,
                    &known_admins,
                )),
            },
            Command::Admin(_) => Ok(AuthLogic::is_admin(username, &known_admins)),
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_merge_command_results() {
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

        let merged = CommandExecutor::merge_command_results(results);
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
        let results = vec![
            CommandExecutionResult::builder().ignored().build(),
            CommandExecutionResult::builder().ignored().build(),
        ];

        let merged = CommandExecutor::merge_command_results(results);
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
