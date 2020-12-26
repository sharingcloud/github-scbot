//! Webhook logic tests

use crate::webhook::constants::ENV_BOT_USERNAME;
use crate::webhook::logic::commands::{parse_command_string_from_comment_line, CommentAction};

const TEST_BOT_USERNAME: &str = "SC-GitBot-Test";

#[actix_rt::test]
async fn test_parse_command_string_from_comment_line() {
    std::env::set_var(ENV_BOT_USERNAME, TEST_BOT_USERNAME);

    assert_eq!(
        parse_command_string_from_comment_line(&format!(
            "{} this-is-a-command",
            std::env::var(ENV_BOT_USERNAME).expect("Empty bot username")
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
        Some(CommentAction::AutoMergeStatus(true))
    );
    assert_eq!(
        CommentAction::from_comment("automerge-", &Vec::new()),
        Some(CommentAction::AutoMergeStatus(false))
    );
    assert_eq!(
        CommentAction::from_comment("this-is-a-command", &Vec::new()),
        None
    );
}
