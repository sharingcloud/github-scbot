//! Webhook logic tests

use github_scbot_core::constants::ENV_BOT_USERNAME;

use super::test_init;
use crate::commands::{parse_command_string_from_comment_line, CommentAction};

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
        parse_command_string_from_comment_line("this-is-a-command"),
        None
    )
}

#[actix_rt::test]
async fn test_comment_action_from_comment() {
    test_init();

    assert_eq!(
        CommentAction::from_comment("noqa+", &Vec::new()),
        Some(CommentAction::SkipQAStatus(true))
    );
    assert_eq!(
        CommentAction::from_comment("noqa-", &Vec::new()),
        Some(CommentAction::SkipQAStatus(false))
    );
    assert_eq!(
        CommentAction::from_comment("qa+", &Vec::new()),
        Some(CommentAction::QAStatus(true))
    );
    assert_eq!(
        CommentAction::from_comment("qa-", &Vec::new()),
        Some(CommentAction::QAStatus(false))
    );
    assert_eq!(
        CommentAction::from_comment("automerge+", &Vec::new()),
        Some(CommentAction::Automerge(true))
    );
    assert_eq!(
        CommentAction::from_comment("automerge-", &Vec::new()),
        Some(CommentAction::Automerge(false))
    );
    assert_eq!(
        CommentAction::from_comment("this-is-a-command", &Vec::new()),
        None
    );
}
