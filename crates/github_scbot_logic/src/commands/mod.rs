//! Commands module.

mod command;
mod handlers;
mod parser;

use github_scbot_api::{
    adapter::IAPIAdapter,
    comments::{add_reaction_to_comment, post_comment},
};
use github_scbot_conf::Config;
use github_scbot_database::models::{IDatabaseAdapter, PullRequestModel, RepositoryModel};
use github_scbot_redis::IRedisAdapter;
use github_scbot_types::issues::GhReactionType;
pub use handlers::handle_qa_command;
pub use parser::CommandParser;
use tracing::info;

pub use self::command::{AdminCommand, Command, CommandResult, UserCommand};
use super::{errors::Result, status::update_pull_request_status};
use crate::{
    auth::{has_right_on_pull_request, is_admin, list_known_admin_usernames},
    commands::command::{CommandExecutionResult, CommandHandlingStatus, ResultAction},
};

/// Command executor.
pub struct CommandExecutor;

impl CommandExecutor {
    /// Execute multiple commands.
    #[allow(clippy::too_many_arguments)]
    pub async fn execute_commands(
        config: &Config,
        api_adapter: &impl IAPIAdapter,
        db_adapter: &dyn IDatabaseAdapter,
        redis_adapter: &dyn IRedisAdapter,
        repo_model: &mut RepositoryModel,
        pr_model: &mut PullRequestModel,
        comment_id: u64,
        comment_author: &str,
        commands: Vec<CommandResult<Command>>,
    ) -> Result<()> {
        let mut status = vec![];

        for command in commands {
            match command {
                Ok(command) => {
                    status.push(
                        Self::execute_command(
                            config,
                            api_adapter,
                            db_adapter,
                            repo_model,
                            pr_model,
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
            repo_model,
            pr_model,
            comment_id,
            command_result,
        )
        .await?;

        Ok(())
    }

    /// Process command result.
    pub async fn process_command_result(
        api_adapter: &impl IAPIAdapter,
        db_adapter: &dyn IDatabaseAdapter,
        redis_adapter: &dyn IRedisAdapter,
        repo_model: &RepositoryModel,
        pr_model: &mut PullRequestModel,
        comment_id: u64,
        command_result: CommandExecutionResult,
    ) -> Result<()> {
        if command_result.should_update_status {
            let sha = api_adapter
                .pulls_get(&repo_model.owner, &repo_model.name, pr_model.get_number())
                .await?
                .head
                .sha;
            update_pull_request_status(
                api_adapter,
                db_adapter,
                redis_adapter,
                repo_model,
                pr_model,
                &sha,
            )
            .await?;
        }

        for action in command_result.result_actions {
            match action {
                ResultAction::AddReaction(reaction) => {
                    add_reaction_to_comment(
                        api_adapter,
                        &repo_model.owner,
                        &repo_model.name,
                        comment_id,
                        reaction,
                    )
                    .await?;
                }
                ResultAction::PostComment(comment) => {
                    post_comment(
                        api_adapter,
                        &repo_model.owner,
                        &repo_model.name,
                        pr_model.get_number(),
                        &comment,
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
    #[allow(clippy::too_many_lines)]
    pub async fn execute_command(
        config: &Config,
        api_adapter: &impl IAPIAdapter,
        db_adapter: &dyn IDatabaseAdapter,
        repo_model: &mut RepositoryModel,
        pr_model: &mut PullRequestModel,
        comment_author: &str,
        command: Command,
    ) -> Result<CommandExecutionResult> {
        let mut command_result: CommandExecutionResult;

        info!(
            command = ?command,
            comment_author = comment_author,
            repository_path = %repo_model.get_path(),
            pull_request_number = pr_model.get_number(),
            message = "Interpreting command"
        );

        if Self::validate_user_rights_on_command(db_adapter, comment_author, pr_model, &command)
            .await?
        {
            command_result = match &command {
                Command::User(cmd) => match cmd {
                    UserCommand::Automerge(s) => {
                        handlers::handle_auto_merge_command(
                            db_adapter,
                            pr_model,
                            comment_author,
                            *s,
                        )
                        .await?
                    }
                    UserCommand::SkipQaStatus(s) => {
                        handlers::handle_skip_qa_command(db_adapter, pr_model, *s).await?
                    }
                    UserCommand::QaStatus(s) => {
                        handlers::handle_qa_command(db_adapter, pr_model, comment_author, *s)
                            .await?
                    }
                    UserCommand::Lock(s, reason) => {
                        handlers::handle_lock_command(
                            db_adapter,
                            pr_model,
                            comment_author,
                            *s,
                            reason.clone(),
                        )
                        .await?
                    }
                    UserCommand::Ping => handlers::handle_ping_command(comment_author)?,
                    UserCommand::Merge => {
                        handlers::handle_merge_command(
                            api_adapter,
                            db_adapter,
                            repo_model,
                            pr_model,
                            comment_author,
                        )
                        .await?
                    }
                    UserCommand::AssignRequiredReviewers(reviewers) => {
                        handlers::handle_assign_required_reviewers_command(
                            api_adapter,
                            db_adapter,
                            repo_model,
                            pr_model,
                            reviewers.clone(),
                        )
                        .await?
                    }
                    UserCommand::UnassignRequiredReviewers(reviewers) => {
                        handlers::handle_unassign_required_reviewers_command(
                            api_adapter,
                            db_adapter,
                            repo_model,
                            pr_model,
                            reviewers.clone(),
                        )
                        .await?
                    }
                    UserCommand::Gif(terms) => {
                        handlers::handle_gif_command(config, api_adapter, &terms).await?
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
                            repo_model,
                            pr_model,
                        )
                        .await?
                    }
                    AdminCommand::Synchronize => {
                        handlers::handle_admin_sync_command(
                            config,
                            api_adapter,
                            db_adapter,
                            repo_model,
                            pr_model,
                        )
                        .await?
                    }
                    AdminCommand::ResetReviews => {
                        handlers::handle_admin_reset_reviews_command(
                            api_adapter,
                            db_adapter,
                            repo_model,
                            pr_model,
                        )
                        .await?
                    }
                    AdminCommand::SetDefaultNeededReviewers(count) => {
                        handlers::handle_set_default_needed_reviewers_command(
                            db_adapter, repo_model, *count,
                        )
                        .await?
                    }
                    AdminCommand::SetDefaultMergeStrategy(strategy) => {
                        handlers::handle_set_default_merge_strategy_command(
                            db_adapter, repo_model, *strategy,
                        )
                        .await?
                    }
                    AdminCommand::SetDefaultPRTitleRegex(rgx) => {
                        handlers::handle_set_default_pr_title_regex_command(
                            db_adapter,
                            repo_model,
                            rgx.clone(),
                        )
                        .await?
                    }
                    AdminCommand::SetNeededReviewers(count) => {
                        handlers::handle_set_needed_reviewers_command(db_adapter, pr_model, *count)
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
        db_adapter: &dyn IDatabaseAdapter,
        username: &str,
        pr_model: &PullRequestModel,
        command: &Command,
    ) -> Result<bool> {
        let known_admins = list_known_admin_usernames(db_adapter).await?;

        match command {
            Command::User(cmd) => match cmd {
                UserCommand::Ping | UserCommand::Help | UserCommand::Gif(_) => Ok(true),
                _ => Ok(has_right_on_pull_request(username, pr_model, &known_admins)),
            },
            Command::Admin(_) => Ok(is_admin(username, &known_admins)),
        }
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_database::{
        models::{AccountModel, DatabaseAdapter},
        tests::using_test_db,
        Result,
    };
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::LogicError;

    #[actix_rt::test]
    async fn test_validate_user_rights_on_command() -> Result<()> {
        using_test_db("test_logic_commands", |config, pool| async move {
            let db_adapter = DatabaseAdapter::new(&pool);

            let creator = "me";
            let repo = RepositoryModel::builder(&config, "me", "test")
                .create_or_update(db_adapter.repository())
                .await
                .unwrap();

            let pr = PullRequestModel::builder(&repo, 1, creator)
                .create_or_update(db_adapter.pull_request())
                .await
                .unwrap();

            AccountModel::builder("non-admin")
                .admin(false)
                .create_or_update(db_adapter.account())
                .await
                .unwrap();

            AccountModel::builder("admin")
                .admin(true)
                .create_or_update(db_adapter.account())
                .await
                .unwrap();

            // PR creator should be valid
            assert_eq!(
                CommandExecutor::validate_user_rights_on_command(
                    &db_adapter,
                    creator,
                    &pr,
                    &Command::User(UserCommand::Merge)
                )
                .await
                .unwrap(),
                true
            );
            // Non-admin should be invalid
            assert_eq!(
                CommandExecutor::validate_user_rights_on_command(
                    &db_adapter,
                    "non-admin",
                    &pr,
                    &Command::User(UserCommand::Merge)
                )
                .await
                .unwrap(),
                false
            );
            // Admin should be valid
            assert_eq!(
                CommandExecutor::validate_user_rights_on_command(
                    &db_adapter,
                    "admin",
                    &pr,
                    &Command::User(UserCommand::Merge)
                )
                .await
                .unwrap(),
                true
            );

            Ok::<_, LogicError>(())
        })
        .await
    }

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
