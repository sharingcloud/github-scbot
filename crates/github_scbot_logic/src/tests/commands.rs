//! Webhook logic tests

use github_scbot_core::constants::ENV_BOT_USERNAME;

use super::test_init;
use crate::commands::{parse_command_string_from_comment_line, Command};

#[actix_rt::test]
async fn test_parse_command_string_from_comment_line() {
    test_init();

    assert_eq!(
        parse_command_string_from_comment_line(&format!(
            "{} this-is-a-command",
            std::env::var(ENV_BOT_USERNAME).unwrap()
        )),
        Some(("this-is-a-command", vec![]))
    );

    assert_eq!(
        parse_command_string_from_comment_line(&format!(
            "{} lock+ Because I choosed to",
            std::env::var(ENV_BOT_USERNAME).unwrap()
        )),
        Some(("lock+", vec!["Because", "I", "choosed", "to"]))
    );

    assert_eq!(
        parse_command_string_from_comment_line("this-is-a-command"),
        None
    )
}

#[actix_rt::test]
async fn test_command_from_comment() {
    test_init();

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
