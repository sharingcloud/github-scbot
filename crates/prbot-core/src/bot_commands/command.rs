use prbot_config::Config;
use prbot_ghapi_interface::types::GhReactionType;
use prbot_models::{MergeStrategy, RuleBranch};
use thiserror::Error;

const MAX_REVIEWERS_PER_COMMAND: usize = 16;

/// Command error.
#[derive(Debug, Error)]
pub enum CommandError {
    /// Unknown command.
    #[error("This command is unknown.")]
    UnknownCommand { command: String },
    /// Argument parsing error.
    #[error("Error while parsing command arguments.")]
    ArgumentParsingError,
    /// Incomplete command.
    #[error("Incomplete command.")]
    IncompleteCommand,
    /// Invalid usage.
    #[error("Invalid usage: {}", usage)]
    InvalidUsage { usage: String },
}

/// Command result.
pub type CommandResult<T> = core::result::Result<T, CommandError>;

/// Command handling status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
#[derive(Debug, PartialEq, Eq)]
pub enum UserCommand {
    /// Skip QA status.
    SkipQaStatus(bool),
    /// Enable/Disable QA status.
    QaStatus(Option<bool>),
    /// Skip checks status.
    SkipChecksStatus(bool),
    /// Enable/Disable automerge.
    Automerge(bool),
    /// Assign reviewers.
    AssignReviewers(Vec<String>),
    /// Assign required reviewers.
    AssignRequiredReviewers(Vec<String>),
    /// Unassign reviewers.
    UnassignReviewers(Vec<String>),
    /// Set merge strategy.
    SetMergeStrategy(MergeStrategy),
    /// Unset merge strategy.
    UnsetMergeStrategy,
    /// Set labels.
    SetLabels(Vec<String>),
    /// Unset labels.
    UnsetLabels(Vec<String>),
    /// Add/Remove lock with optional reason.
    Lock(bool, Option<String>),
    /// Post a random gif.
    Gif(String),
    /// Merge pull request.
    Merge(Option<MergeStrategy>),
    /// Ping the bot.
    Ping,
    /// Show help message.
    Help,
    /// Is admin?
    IsAdmin,
}

/// Admin command.
#[derive(Debug, PartialEq, Eq)]
pub enum AdminCommand {
    /// Show admin help message.
    Help,
    /// Synchronize status.
    Synchronize,
    /// Enable bot on pull request (used with manual interaction).
    Enable,
    /// Disable bot on pull request (used with manual interaction).
    Disable,
    /// Reset summary comment.
    ResetSummary,
    /// Add merge rule.
    AddMergeRule(RuleBranch, RuleBranch, MergeStrategy),
    /// Set default needed reviewers count.
    SetDefaultNeededReviewers(u64),
    /// Set default merge strategy.
    SetDefaultMergeStrategy(MergeStrategy),
    /// Set default PR title validation regex.
    SetDefaultPrTitleRegex(String),
    /// Set default automerge status.
    SetDefaultAutomerge(bool),
    /// Set default QA status.
    SetDefaultQaStatus(bool),
    /// Set default checks status.
    SetDefaultChecksStatus(bool),
    /// Set needed reviewers count.
    SetNeededReviewers(u64),
}

/// Command.
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    /// User command.
    User(UserCommand),
    /// Admin command.
    Admin(AdminCommand),
}

/// Command execution result.
#[derive(Debug, PartialEq, Eq)]
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
#[derive(Debug, PartialEq, Eq)]
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
            "r+" => Self::User(UserCommand::AssignReviewers(Self::parse_reviewers(args)?)),
            "req+" => Self::User(UserCommand::AssignRequiredReviewers(Self::parse_reviewers(
                args,
            )?)),
            "r-" => Self::User(UserCommand::UnassignReviewers(Self::parse_reviewers(args)?)),
            "strategy+" => Self::User(UserCommand::SetMergeStrategy(Self::parse_merge_strategy(
                args,
            )?)),
            "strategy-" => Self::User(UserCommand::UnsetMergeStrategy),
            "gif" => Self::User(UserCommand::Gif(Self::parse_text(args))),
            "merge" => Self::User(UserCommand::Merge(Self::parse_optional_merge_strategy(
                args,
            )?)),
            "labels+" => Self::User(UserCommand::SetLabels(Self::parse_labels(args)?)),
            "labels-" => Self::User(UserCommand::UnsetLabels(Self::parse_labels(args)?)),
            "ping" => Self::User(UserCommand::Ping),
            "is-admin" => Self::User(UserCommand::IsAdmin),
            "help" => Self::User(UserCommand::Help),
            // Admin commands
            "admin-help" => Self::Admin(AdminCommand::Help),
            "admin-sync" => Self::Admin(AdminCommand::Synchronize),
            "admin-reset-summary" => Self::Admin(AdminCommand::ResetSummary),
            "admin-enable" => Self::Admin(AdminCommand::Enable),
            "admin-disable" => Self::Admin(AdminCommand::Disable),
            "admin-add-merge-rule" => {
                let (base, head, strategy) = Self::parse_merge_rule(args)?;
                Self::Admin(AdminCommand::AddMergeRule(base, head, strategy))
            }
            "admin-set-default-needed-reviewers" => Self::Admin(
                AdminCommand::SetDefaultNeededReviewers(Self::parse_u64(args)?),
            ),
            "admin-set-default-merge-strategy" => Self::Admin(
                AdminCommand::SetDefaultMergeStrategy(Self::parse_merge_strategy(args)?),
            ),
            "admin-set-default-pr-title-regex" => {
                Self::Admin(AdminCommand::SetDefaultPrTitleRegex(Self::parse_text(args)))
            }
            "admin-set-default-checks-status+" => {
                Self::Admin(AdminCommand::SetDefaultChecksStatus(true))
            }
            "admin-set-default-checks-status-" => {
                Self::Admin(AdminCommand::SetDefaultChecksStatus(false))
            }
            "admin-set-default-qa-status+" => Self::Admin(AdminCommand::SetDefaultQaStatus(true)),
            "admin-set-default-qa-status-" => Self::Admin(AdminCommand::SetDefaultQaStatus(false)),
            "admin-set-default-automerge+" => Self::Admin(AdminCommand::SetDefaultAutomerge(true)),
            "admin-set-default-automerge-" => Self::Admin(AdminCommand::SetDefaultAutomerge(false)),
            "admin-set-needed-reviewers" => {
                Self::Admin(AdminCommand::SetNeededReviewers(Self::parse_u64(args)?))
            }
            // Unknown command
            unknown => {
                return Err(CommandError::UnknownCommand {
                    command: unknown.to_string(),
                })
            }
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
                AdminCommand::AddMergeRule(base, head, strategy) => {
                    format!("admin-add-merge-rule {} {} {}", base, head, strategy)
                }
                AdminCommand::SetDefaultMergeStrategy(strategy) => {
                    format!("admin-set-default-merge-strategy {}", strategy)
                }
                AdminCommand::SetDefaultNeededReviewers(count) => {
                    format!("admin-set-default-needed-reviewers {}", count)
                }
                AdminCommand::SetDefaultPrTitleRegex(rgx) => {
                    format!("admin-set-default-pr-title-regex {}", rgx)
                }
                AdminCommand::SetDefaultChecksStatus(status) => {
                    format!(
                        "admin-set-default-checks-status{}",
                        Self::plus_minus(*status)
                    )
                }
                AdminCommand::SetDefaultQaStatus(status) => {
                    format!("admin-set-default-qa-status{}", Self::plus_minus(*status))
                }
                AdminCommand::SetNeededReviewers(count) => {
                    format!("admin-set-needed-reviewers {}", count)
                }
                AdminCommand::SetDefaultAutomerge(status) => {
                    format!("admin-set-default-automerge{}", Self::plus_minus(*status))
                }
                AdminCommand::Synchronize => "admin-sync".into(),
                AdminCommand::ResetSummary => "admin-reset-summary".into(),
            },
            Self::User(cmd) => match cmd {
                UserCommand::AssignReviewers(reviewers) => {
                    format!("r+ {}", reviewers.join(" "))
                }
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
                        format!("merge {}", strategy)
                    } else {
                        "merge".into()
                    }
                }
                UserCommand::SetMergeStrategy(strategy) => {
                    format!("strategy+ {}", strategy)
                }
                UserCommand::UnsetMergeStrategy => "strategy-".into(),
                UserCommand::SetLabels(labels) => format!("labels+ {}", labels.join(" ")),
                UserCommand::UnsetLabels(labels) => format!("labels- {}", labels.join(" ")),
                UserCommand::Ping => "ping".into(),
                UserCommand::QaStatus(status) => format!("qa{}", Self::plus_minus_option(*status)),
                UserCommand::SkipQaStatus(status) => {
                    format!("noqa{}", Self::plus_minus(*status))
                }
                UserCommand::SkipChecksStatus(status) => {
                    format!("nochecks{}", Self::plus_minus(*status))
                }
                UserCommand::UnassignReviewers(reviewers) => {
                    format!("r- {}", reviewers.join(" "))
                }
            },
        }
    }

    fn parse_u64(args: &[&str]) -> CommandResult<u64> {
        args.join(" ")
            .parse()
            .map_err(|_e| CommandError::ArgumentParsingError)
    }

    fn parse_merge_strategy(args: &[&str]) -> CommandResult<MergeStrategy> {
        MergeStrategy::try_from(&args.join(" ")[..])
            .map_err(|_e| CommandError::ArgumentParsingError)
    }

    fn parse_optional_merge_strategy(args: &[&str]) -> CommandResult<Option<MergeStrategy>> {
        let args = &args.join(" ");
        if args.trim().is_empty() {
            Ok(None)
        } else {
            Ok(Some(
                MergeStrategy::try_from(&args[..])
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

    fn parse_labels(labels: &[&str]) -> CommandResult<Vec<String>> {
        let labels = labels
            .iter()
            .map(|x| x.trim().to_string())
            .collect::<Vec<_>>();

        if labels.is_empty() {
            Err(CommandError::IncompleteCommand)
        } else {
            Ok(labels)
        }
    }

    fn parse_reviewers(reviewers: &[&str]) -> CommandResult<Vec<String>> {
        let reviewers: Vec<String> = reviewers
            .iter()
            .map(|x| x.trim_matches('@').to_string())
            .collect();

        if reviewers.is_empty() {
            Err(CommandError::IncompleteCommand)
        } else if reviewers.len() > MAX_REVIEWERS_PER_COMMAND {
            Err(CommandError::InvalidUsage {
                usage: format!(
                    "You can only specify up to {MAX_REVIEWERS_PER_COMMAND} reviewers on one command."
                )
            })
        } else {
            Ok(reviewers)
        }
    }

    fn parse_merge_rule(args: &[&str]) -> CommandResult<(RuleBranch, RuleBranch, MergeStrategy)> {
        if args.len() != 3 {
            return Err(CommandError::IncompleteCommand);
        }

        let base = RuleBranch::from(args[0]);
        let head = RuleBranch::from(args[1]);
        let strategy =
            MergeStrategy::try_from(args[2]).map_err(|_| CommandError::ArgumentParsingError)?;

        Ok((base, head, strategy))
    }

    /// Convert to bot string.
    pub fn to_bot_string(&self, config: &Config) -> String {
        format!(
            "{bot} {command}",
            bot = config.name,
            command = self.to_command_string()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_u32() {
        assert!(matches!(Command::parse_u64(&["123"]), Ok(123)));
        assert!(matches!(
            Command::parse_u64(&["123", "456"]),
            Err(CommandError::ArgumentParsingError)
        ));
        assert!(matches!(
            Command::parse_u64(&["toto"]),
            Err(CommandError::ArgumentParsingError)
        ));
    }

    #[test]
    fn test_parse_merge_strategy() {
        assert!(matches!(
            Command::parse_merge_strategy(&["merge"]),
            Ok(MergeStrategy::Merge)
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

    #[test]
    fn test_parse_merge_rule() {
        assert_eq!(
            Command::parse_merge_rule(&["*", "stable", "squash"]).unwrap(),
            (
                RuleBranch::Wildcard,
                RuleBranch::Named("stable".into()),
                MergeStrategy::Squash
            )
        )
    }
}
