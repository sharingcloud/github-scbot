//! Webhook logic tests

use super::test_config;
use crate::commands::{parse_command_string_from_comment_line, Command};

#[actix_rt::test]
async fn test_parse_command_string_from_comment_line() {
    let config = test_config();

    assert_eq!(
        parse_command_string_from_comment_line(
            &config,
            &format!("{} this-is-a-command", config.bot_username)
        ),
        Some(("this-is-a-command", vec![]))
    );

    assert_eq!(
        parse_command_string_from_comment_line(
            &config,
            &format!("{} lock+ Because I choosed to", config.bot_username)
        ),
        Some(("lock+", vec!["Because", "I", "choosed", "to"]))
    );

    assert_eq!(
        parse_command_string_from_comment_line(&config, "this-is-a-command"),
        None
    )
}

#[test]
fn test_command_from_comment() {
    assert_eq!(
        Command::from_comment("noqa+", &Vec::new()),
        Some(Command::SkipQAStatus(true))
    );
    assert_eq!(
        Command::from_comment("noqa-", &Vec::new()),
        Some(Command::SkipQAStatus(false))
    );
    assert_eq!(
        Command::from_comment("qa+", &Vec::new()),
        Some(Command::QAStatus(Some(true)))
    );
    assert_eq!(
        Command::from_comment("qa-", &Vec::new()),
        Some(Command::QAStatus(Some(false)))
    );
    assert_eq!(
        Command::from_comment("qa?", &Vec::new()),
        Some(Command::QAStatus(None))
    );
    assert_eq!(
        Command::from_comment("automerge+", &Vec::new()),
        Some(Command::Automerge(true))
    );
    assert_eq!(
        Command::from_comment("automerge-", &Vec::new()),
        Some(Command::Automerge(false))
    );
    assert_eq!(
        Command::from_comment("this-is-a-command", &Vec::new()),
        None
    );
}
