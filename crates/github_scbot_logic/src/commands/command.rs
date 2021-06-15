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

/// Command.
#[derive(Debug, PartialEq)]
pub enum Command {
    /// Skip QA status.
    SkipQaStatus(bool),
    /// Enable/Disable QA status.
    QaStatus(Option<bool>),
    /// Enable/Disable automerge.
    Automerge(bool),
    /// Assign required reviewers.
    AssignRequiredReviewers(Vec<String>),
    /// Unassign required reviewers.
    UnassignRequiredReviewers(Vec<String>),
    /// Add/Remove lock with optional reason.
    Lock(bool, Option<String>),
    /// Post a random gif.
    Gif(String),
    /// Merge pull request.
    Merge,
    /// Ping the bot.
    Ping,
    /// Show help message.
    Help,
    /// Is admin?
    IsAdmin,
    /// Show admin help message.
    AdminHelp,
    /// Synchronize status.
    AdminSynchronize,
    /// Reset reviews.
    AdminResetReviews,
    /// Enable bot on pull request (used with manual interaction).
    AdminEnable,
    /// Disable bot on pull request (used with manual interaction).
    AdminDisable,
    /// Set default needed reviewers count.
    AdminSetDefaultNeededReviewers(u32),
    /// Set default merge strategy.
    AdminSetDefaultMergeStrategy(GhMergeStrategy),
    /// Set default PR title validation regex.
    AdminSetDefaultPRTitleRegex(String),
    /// Set needed reviewers count.
    AdminSetNeededReviewers(u32),
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
            "noqa+" => Self::SkipQaStatus(true),
            "noqa-" => Self::SkipQaStatus(false),
            "qa+" => Self::QaStatus(Some(true)),
            "qa-" => Self::QaStatus(Some(false)),
            "qa?" => Self::QaStatus(None),
            "automerge+" => Self::Automerge(true),
            "automerge-" => Self::Automerge(false),
            "lock+" => Self::Lock(true, Self::parse_message(args)),
            "lock-" => Self::Lock(false, Self::parse_message(args)),
            "req+" => Self::AssignRequiredReviewers(Self::parse_reviewers(args)?),
            "req-" => Self::UnassignRequiredReviewers(Self::parse_reviewers(args)?),
            "gif" => Self::Gif(Self::parse_text(args)),
            "merge" => Self::Merge,
            "ping" => Self::Ping,
            "is-admin" => Self::IsAdmin,
            "help" => Self::Help,
            // Admin commands
            "admin-help" => Self::AdminHelp,
            "admin-sync" => Self::AdminSynchronize,
            "admin-reset-reviews" => Self::AdminResetReviews,
            "admin-enable" => Self::AdminEnable,
            "admin-disable" => Self::AdminDisable,
            "admin-set-default-needed-reviewers" => {
                Self::AdminSetDefaultNeededReviewers(Self::parse_u32(args)?)
            }
            "admin-set-default-merge-strategy" => {
                Self::AdminSetDefaultMergeStrategy(Self::parse_merge_strategy(args)?)
            }
            "admin-set-default-pr-title-regex" => {
                Self::AdminSetDefaultPRTitleRegex(Self::parse_text(args))
            }
            "admin-set-needed-reviewers" => Self::AdminSetNeededReviewers(Self::parse_u32(args)?),
            // Unknown command
            unknown => return Err(CommandError::UnknownCommand(unknown.into())),
        }))
    }

    fn to_command_string(&self) -> String {
        match self {
            Self::AdminEnable => "admin-enable".into(),
            Self::AdminDisable => "admin-disable".into(),
            Self::AdminHelp => "admin-help".into(),
            Self::AdminSetDefaultMergeStrategy(strategy) => {
                format!("admin-set-default-merge-strategy {}", strategy.to_string())
            }
            Self::AdminSetDefaultNeededReviewers(count) => {
                format!("admin-set-default-needed-reviewers {}", count)
            }
            Self::AdminSetDefaultPRTitleRegex(rgx) => {
                format!("admin-set-default-pr-title-regex {}", rgx.to_string())
            }
            Self::AdminSetNeededReviewers(count) => format!("admin-set-needed-reviewers {}", count),
            Self::AdminSynchronize => "admin-sync".into(),
            Self::AdminResetReviews => "admin-reset-reviews".into(),
            Self::AssignRequiredReviewers(reviewers) => format!("req+ {}", reviewers.join(" ")),
            Self::Automerge(status) => format!("automerge{}", if *status { "+" } else { "-" }),
            Self::Gif(search) => format!("gif {}", search),
            Self::Help => "help".into(),
            Self::IsAdmin => "is-admin".into(),
            Self::Lock(status, reason) => {
                let mut lock = format!("lock{}", if *status { "+" } else { "-" });
                if let Some(reason) = reason {
                    lock = format!("{} {}", lock, reason);
                }
                lock
            }
            Self::Merge => "merge".into(),
            Self::Ping => "ping".into(),
            Self::QaStatus(status) => format!(
                "qa{}",
                match status {
                    None => "?",
                    Some(true) => "+",
                    Some(false) => "-",
                }
            ),
            Self::SkipQaStatus(status) => format!("noqa{}", if *status { "+" } else { "-" }),
            Self::UnassignRequiredReviewers(reviewers) => format!("req- {}", reviewers.join(" ")),
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
            .filter_map(|x| x.strip_prefix('@').map(str::to_string))
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
                "@four".to_string()
            ]
        );

        assert!(matches!(
            Command::parse_reviewers(&["toto"]),
            Err(CommandError::IncompleteCommand)
        ));
        assert!(matches!(
            Command::parse_reviewers(&[]),
            Err(CommandError::IncompleteCommand)
        ));
    }
}
