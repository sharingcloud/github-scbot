use std::convert::TryFrom;

use github_scbot_conf::Config;
use github_scbot_types::{issues::GhReactionType, pulls::GhMergeStrategy};
use smart_default::SmartDefault;
use thiserror::Error;

/// Command error.
#[derive(Debug, Error, PartialEq)]
pub enum CommandError {
    /// Unknown command.
    #[error("This command is unknown.")]
    UnknownCommand(String),
    /// Argument parsing error.
    #[error("Error while parsing command arguments.")]
    ArgumentParsingError,
    /// Incomplete command.
    #[error("Incomplete command.")]
    IncompleteCommand,
}

/// Command result.
pub type CommandResult<T> = core::result::Result<T, CommandError>;

/// Command handling status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, SmartDefault)]
pub enum CommandHandlingStatus {
    /// Command handled.
    #[default]
    Handled,
    /// Command denied.
    Denied,
    /// Command ignored.
    Ignored,
}

/// User command.
#[derive(Debug, PartialEq)]
pub enum UserCommand {
    /// Skip QA status.
    SkipQaStatus(bool),
    /// Enable/Disable QA status.
    QaStatus(Option<bool>),
    /// Skip checks status.
    SkipChecksStatus(bool),
    /// Enable/Disable automerge.
    Automerge(bool),
    /// Assign required reviewers.
    AssignRequiredReviewers(Vec<String>),
    /// Unassign required reviewers.
    UnassignRequiredReviewers(Vec<String>),
    /// Set merge strategy.
    SetMergeStrategy(GhMergeStrategy),
    /// Unset merge strategy.
    UnsetMergeStrategy,
    /// Add/Remove lock with optional reason.
    Lock(bool, Option<String>),
    /// Post a random gif.
    Gif(String),
    /// Merge pull request.
    Merge(Option<GhMergeStrategy>),
    /// Ping the bot.
    Ping,
    /// Show help message.
    Help,
    /// Is admin?
    IsAdmin,
}

/// Admin command.
#[derive(Debug, PartialEq)]
pub enum AdminCommand {
    /// Show admin help message.
    Help,
    /// Synchronize status.
    Synchronize,
    /// Reset reviews.
    ResetReviews,
    /// Enable bot on pull request (used with manual interaction).
    Enable,
    /// Disable bot on pull request (used with manual interaction).
    Disable,
    /// Reset summary comment.
    ResetSummary,
    /// Set default needed reviewers count.
    SetDefaultNeededReviewers(u32),
    /// Set default merge strategy.
    SetDefaultMergeStrategy(GhMergeStrategy),
    /// Set default PR title validation regex.
    SetDefaultPRTitleRegex(String),
    /// Set default automerge status.
    SetDefaultAutomerge(bool),
    /// Set default QA status.
    SetDefaultQAStatus(bool),
    /// Set default checks status.
    SetDefaultChecksStatus(bool),
    /// Set needed reviewers count.
    SetNeededReviewers(u32),
}

/// Command.
#[derive(Debug, PartialEq)]
pub enum Command {
    /// User command.
    User(UserCommand),
    /// Admin command.
    Admin(AdminCommand),
}

/// Command execution result.
#[derive(Debug, PartialEq)]
pub struct CommandExecutionResult {
    /// Should update status.
    pub should_update_status: bool,
    /// Handling status.
    pub handling_status: CommandHandlingStatus,
    /// Actions.
    pub result_actions: Vec<ResultAction>,
}

impl CommandExecutionResult {
    /// Create builder instance.
    pub fn builder() -> CommandExecutionResultBuilder {
        CommandExecutionResultBuilder::default()
    }
}

/// Command execution result builder.
#[derive(Debug, Default)]
pub struct CommandExecutionResultBuilder {
    should_update_status: bool,
    handling_status: CommandHandlingStatus,
    result_actions: Vec<ResultAction>,
}

impl CommandExecutionResultBuilder {
    /// Set status update.
    pub fn with_status_update(mut self, value: bool) -> Self {
        self.should_update_status = value;
        self
    }

    /// Set ignored result.
    pub fn ignored(mut self) -> Self {
        self.handling_status = CommandHandlingStatus::Ignored;
        self
    }

    /// Set denied result.
    pub fn denied(mut self) -> Self {
        self.handling_status = CommandHandlingStatus::Denied;
        self
    }

    /// Set handled result.
    pub fn handled(mut self) -> Self {
        self.handling_status = CommandHandlingStatus::Handled;
        self
    }

    /// Add result action.
    pub fn with_action(mut self, action: ResultAction) -> Self {
        self.result_actions.push(action);
        self
    }

    /// Add multiple result actions.
    pub fn with_actions(mut self, actions: Vec<ResultAction>) -> Self {
        self.result_actions.extend(actions);
        self
    }

    /// Build execution result.
    pub fn build(self) -> CommandExecutionResult {
        CommandExecutionResult {
            handling_status: self.handling_status,
            result_actions: self.result_actions,
            should_update_status: self.should_update_status,
        }
    }
}

/// Result action.
#[derive(Debug, PartialEq)]
pub enum ResultAction {
    /// Post comment.
    PostComment(String),
    /// Add reaction.
    AddReaction(GhReactionType),
}

impl Command {
    /// Create a command from a comment and arguments.
    pub fn from_comment(comment: &str, args: &[&str]) -> CommandResult<Option<Self>> {
        Ok(Some(match comment {
            "noqa+" => Self::User(UserCommand::SkipQaStatus(true)),
            "noqa-" => Self::User(UserCommand::SkipQaStatus(false)),
            "qa+" => Self::User(UserCommand::QaStatus(Some(true))),
            "qa-" => Self::User(UserCommand::QaStatus(Some(false))),
            "nochecks+" => Self::User(UserCommand::SkipChecksStatus(true)),
            "nochecks-" => Self::User(UserCommand::SkipChecksStatus(false)),
            "qa?" => Self::User(UserCommand::QaStatus(None)),
            "automerge+" => Self::User(UserCommand::Automerge(true)),
            "automerge-" => Self::User(UserCommand::Automerge(false)),
            "lock+" => Self::User(UserCommand::Lock(true, Self::parse_message(args))),
            "lock-" => Self::User(UserCommand::Lock(false, Self::parse_message(args))),
            "req+" => Self::User(UserCommand::AssignRequiredReviewers(Self::parse_reviewers(
                args,
            )?)),
            "req-" => Self::User(UserCommand::UnassignRequiredReviewers(
                Self::parse_reviewers(args)?,
            )),
            "strategy+" => Self::User(UserCommand::SetMergeStrategy(Self::parse_merge_strategy(
                args,
            )?)),
            "strategy-" => Self::User(UserCommand::UnsetMergeStrategy),
            "gif" => Self::User(UserCommand::Gif(Self::parse_text(args))),
            "merge" => Self::User(UserCommand::Merge(Self::parse_optional_merge_strategy(
                args,
            )?)),
            "ping" => Self::User(UserCommand::Ping),
            "is-admin" => Self::User(UserCommand::IsAdmin),
            "help" => Self::User(UserCommand::Help),
            // Admin commands
            "admin-help" => Self::Admin(AdminCommand::Help),
            "admin-sync" => Self::Admin(AdminCommand::Synchronize),
            "admin-reset-reviews" => Self::Admin(AdminCommand::ResetReviews),
            "admin-reset-summary" => Self::Admin(AdminCommand::ResetSummary),
            "admin-enable" => Self::Admin(AdminCommand::Enable),
            "admin-disable" => Self::Admin(AdminCommand::Disable),
            "admin-set-default-needed-reviewers" => Self::Admin(
                AdminCommand::SetDefaultNeededReviewers(Self::parse_u32(args)?),
            ),
            "admin-set-default-merge-strategy" => Self::Admin(
                AdminCommand::SetDefaultMergeStrategy(Self::parse_merge_strategy(args)?),
            ),
            "admin-set-default-pr-title-regex" => {
                Self::Admin(AdminCommand::SetDefaultPRTitleRegex(Self::parse_text(args)))
            }
            "admin-set-default-checks-status+" => {
                Self::Admin(AdminCommand::SetDefaultChecksStatus(true))
            }
            "admin-set-default-checks-status-" => {
                Self::Admin(AdminCommand::SetDefaultChecksStatus(false))
            }
            "admin-set-default-qa-status+" => Self::Admin(AdminCommand::SetDefaultQAStatus(true)),
            "admin-set-default-qa-status-" => Self::Admin(AdminCommand::SetDefaultQAStatus(false)),
            "admin-set-default-automerge+" => Self::Admin(AdminCommand::SetDefaultAutomerge(true)),
            "admin-set-default-automerge-" => Self::Admin(AdminCommand::SetDefaultAutomerge(false)),
            "admin-set-needed-reviewers" => {
                Self::Admin(AdminCommand::SetNeededReviewers(Self::parse_u32(args)?))
            }
            // Unknown command
            unknown => return Err(CommandError::UnknownCommand(unknown.into())),
        }))
    }

    fn plus_minus(status: bool) -> &'static str {
        if status {
            "+"
        } else {
            "-"
        }
    }

    fn plus_minus_option(status: Option<bool>) -> &'static str {
        if let Some(status) = status {
            Self::plus_minus(status)
        } else {
            "?"
        }
    }

    fn to_command_string(&self) -> String {
        match self {
            Self::Admin(cmd) => match cmd {
                AdminCommand::Enable => "admin-enable".into(),
                AdminCommand::Disable => "admin-disable".into(),
                AdminCommand::Help => "admin-help".into(),
                AdminCommand::SetDefaultMergeStrategy(strategy) => {
                    format!("admin-set-default-merge-strategy {}", strategy.to_string())
                }
                AdminCommand::SetDefaultNeededReviewers(count) => {
                    format!("admin-set-default-needed-reviewers {}", count)
                }
                AdminCommand::SetDefaultPRTitleRegex(rgx) => {
                    format!("admin-set-default-pr-title-regex {}", rgx.to_string())
                }
                AdminCommand::SetDefaultChecksStatus(status) => {
                    format!(
                        "admin-set-default-checks-status{}",
                        Self::plus_minus(*status)
                    )
                }
                AdminCommand::SetDefaultQAStatus(status) => {
                    format!("admin-set-default-qa-status{}", Self::plus_minus(*status))
                }
                AdminCommand::SetNeededReviewers(count) => {
                    format!("admin-set-needed-reviewers {}", count)
                }
                AdminCommand::SetDefaultAutomerge(status) => {
                    format!("admin-set-default-automerge{}", Self::plus_minus(*status))
                }
                AdminCommand::Synchronize => "admin-sync".into(),
                AdminCommand::ResetReviews => "admin-reset-reviews".into(),
                AdminCommand::ResetSummary => "admin-reset-summary".into(),
            },
            Self::User(cmd) => match cmd {
                UserCommand::AssignRequiredReviewers(reviewers) => {
                    format!("req+ {}", reviewers.join(" "))
                }
                UserCommand::Automerge(status) => {
                    format!("automerge{}", Self::plus_minus(*status))
                }
                UserCommand::Gif(search) => format!("gif {}", search),
                UserCommand::Help => "help".into(),
                UserCommand::IsAdmin => "is-admin".into(),
                UserCommand::Lock(status, reason) => {
                    let mut lock = format!("lock{}", Self::plus_minus(*status));
                    if let Some(reason) = reason {
                        lock = format!("{} {}", lock, reason);
                    }
                    lock
                }
                UserCommand::Merge(strategy) => {
                    if let Some(strategy) = strategy {
                        format!("merge {}", strategy.to_string())
                    } else {
                        "merge".into()
                    }
                }
                UserCommand::SetMergeStrategy(strategy) => {
                    format!("strategy+ {}", strategy.to_string())
                }
                UserCommand::UnsetMergeStrategy => "strategy-".into(),
                UserCommand::Ping => "ping".into(),
                UserCommand::QaStatus(status) => format!("qa{}", Self::plus_minus_option(*status)),
                UserCommand::SkipQaStatus(status) => {
                    format!("noqa{}", Self::plus_minus(*status))
                }
                UserCommand::SkipChecksStatus(status) => {
                    format!("nochecks{}", Self::plus_minus(*status))
                }
                UserCommand::UnassignRequiredReviewers(reviewers) => {
                    format!("req- {}", reviewers.join(" "))
                }
            },
        }
    }

    fn parse_u32(args: &[&str]) -> CommandResult<u32> {
        args.join(" ")
            .parse()
            .map_err(|_e| CommandError::ArgumentParsingError)
    }

    fn parse_merge_strategy(args: &[&str]) -> CommandResult<GhMergeStrategy> {
        GhMergeStrategy::try_from(&args.join(" ")[..])
            .map_err(|_e| CommandError::ArgumentParsingError)
    }

    fn parse_optional_merge_strategy(args: &[&str]) -> CommandResult<Option<GhMergeStrategy>> {
        let args = &args.join(" ");
        if args.trim().is_empty() {
            Ok(None)
        } else {
            Ok(Some(
                GhMergeStrategy::try_from(&args[..])
                    .map_err(|_e| CommandError::ArgumentParsingError)?,
            ))
        }
    }

    fn parse_message(args: &[&str]) -> Option<String> {
        if args.is_empty() {
            None
        } else {
            Some(args.join(" "))
        }
    }

    fn parse_text(words: &[&str]) -> String {
        words.join(" ")
    }

    fn parse_reviewers(reviewers: &[&str]) -> CommandResult<Vec<String>> {
        let reviewers: Vec<String> = reviewers
            .iter()
            .map(|x| x.trim_matches('@').to_string())
            .collect();

        if reviewers.is_empty() {
            Err(CommandError::IncompleteCommand)
        } else {
            Ok(reviewers)
        }
    }

    /// Convert to bot string.
    pub fn to_bot_string(&self, config: &Config) -> String {
        format!(
            "{bot} {command}",
            bot = config.bot_username,
            command = self.to_command_string()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_u32() {
        assert!(matches!(Command::parse_u32(&["123"]), Ok(123)));
        assert!(matches!(
            Command::parse_u32(&["123", "456"]),
            Err(CommandError::ArgumentParsingError)
        ));
        assert!(matches!(
            Command::parse_u32(&["toto"]),
            Err(CommandError::ArgumentParsingError)
        ));
    }

    #[test]
    fn test_parse_merge_strategy() {
        assert!(matches!(
            Command::parse_merge_strategy(&["merge"]),
            Ok(GhMergeStrategy::Merge)
        ));
        assert!(matches!(
            Command::parse_merge_strategy(&["what"]),
            Err(CommandError::ArgumentParsingError)
        ));
        assert!(matches!(
            Command::parse_merge_strategy(&[]),
            Err(CommandError::ArgumentParsingError)
        ));
    }

    #[test]
    fn test_parse_message() {
        assert_eq!(
            Command::parse_message(&["hello", "world"]),
            Some("hello world".into())
        );
        assert_eq!(Command::parse_message(&[]), None);
    }

    #[test]
    fn test_parse_text() {
        assert_eq!(
            Command::parse_text(&["hello", "world"]),
            "hello world".to_string()
        );
        assert_eq!(Command::parse_text(&[]), "".to_string());
    }

    #[test]
    fn test_parse_reviewers() {
        assert_eq!(
            Command::parse_reviewers(&["@one", "@two", "@three", "@@four", "5"]).unwrap(),
            vec![
                "one".to_string(),
                "two".to_string(),
                "three".to_string(),
                "four".to_string(),
                "5".to_string()
            ]
        );

        assert_eq!(
            Command::parse_reviewers(&["toto"]).unwrap(),
            vec!["toto".to_string(),]
        );
        assert!(matches!(
            Command::parse_reviewers(&[]),
            Err(CommandError::IncompleteCommand)
        ));
    }
}
