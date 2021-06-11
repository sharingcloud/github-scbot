use github_scbot_conf::Config;

use super::command::{Command, CommandResult};

/// Command parser.
pub struct CommandParser;

impl CommandParser {
    /// Parse commands from comment body.
    pub fn parse_commands(config: &Config, comment_body: &str) -> Vec<CommandResult<Command>> {
        let mut commands = vec![];

        for line in comment_body.lines() {
            match Self::parse_single_command(config, line) {
                Err(e) => {
                    commands.push(Err(e));
                }
                Ok(Some(command)) => {
                    commands.push(Ok(command));
                }
                Ok(None) => (),
            }
        }

        commands
    }

    /// Parse command from a single comment line.
    pub fn parse_single_command(config: &Config, line: &str) -> CommandResult<Option<Command>> {
        if let Some((command_line, args)) =
            Self::parse_command_string_from_comment_line(config, line)
        {
            let command = Command::from_comment(command_line, &args)?;
            Ok(command)
        } else {
            Ok(None)
        }
    }

    /// Parse command string from comment line.
    pub fn parse_command_string_from_comment_line<'a>(
        config: &Config,
        comment: &'a str,
    ) -> Option<(&'a str, Vec<&'a str>)> {
        if comment.starts_with(&config.bot_username) {
            // Plus one for the '@' symbol
            let (_, command) = comment.split_at(config.bot_username.len());
            let mut split = command.trim().split_whitespace();

            if let Some(command) = split.next() {
                // Take command and remaining args
                return Some((command, split.collect()));
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_conf::Config;
    use pretty_assertions::assert_eq;

    use crate::commands::{
        command::{Command, CommandError},
        parser::CommandParser,
    };

    fn create_test_config() -> Config {
        let mut config = Config::from_env();
        config.bot_username = "test-bot".into();
        config
    }

    #[test]
    fn test_parse_command_string_from_comment_line() {
        let config = create_test_config();

        assert_eq!(
            CommandParser::parse_command_string_from_comment_line(
                &config,
                &format!("{} this-is-a-command", config.bot_username)
            ),
            Some(("this-is-a-command", vec![]))
        );

        assert_eq!(
            CommandParser::parse_command_string_from_comment_line(
                &config,
                &format!("{} lock+ Because I choosed to", config.bot_username)
            ),
            Some(("lock+", vec!["Because", "I", "choosed", "to"]))
        );

        assert_eq!(
            CommandParser::parse_command_string_from_comment_line(&config, "this-is-a-command"),
            None
        )
    }

    #[test]
    fn test_command_from_comment() {
        assert_eq!(
            Command::from_comment("noqa+", &[]),
            Ok(Some(Command::SkipQaStatus(true)))
        );
        assert_eq!(
            Command::from_comment("noqa-", &[]),
            Ok(Some(Command::SkipQaStatus(false)))
        );
        assert_eq!(
            Command::from_comment("qa+", &[]),
            Ok(Some(Command::QaStatus(Some(true))))
        );
        assert_eq!(
            Command::from_comment("qa-", &[]),
            Ok(Some(Command::QaStatus(Some(false))))
        );
        assert_eq!(
            Command::from_comment("qa?", &[]),
            Ok(Some(Command::QaStatus(None)))
        );
        assert_eq!(
            Command::from_comment("automerge+", &[]),
            Ok(Some(Command::Automerge(true)))
        );
        assert_eq!(
            Command::from_comment("automerge-", &[]),
            Ok(Some(Command::Automerge(false)))
        );
        assert_eq!(
            Command::from_comment("this-is-a-command", &[]),
            Err(CommandError::UnknownCommand("this-is-a-command".into()))
        );
        assert_eq!(
            Command::from_comment("req+", &[]),
            Err(CommandError::IncompleteCommand)
        );
        assert_eq!(
            Command::from_comment("admin-set-needed-reviewers", &["12"]),
            Ok(Some(Command::AdminSetNeededReviewers(12)))
        );
        assert_eq!(
            Command::from_comment("admin-set-needed-reviewers", &["toto"]),
            Err(CommandError::ArgumentParsingError)
        );
    }
}