//! Webhook logic tests

use crate::webhook::logic::parse_command_string_from_comment_line;
use crate::webhook::{constants::ENV_BOT_USERNAME, logic::CommentAction};

const TEST_BOT_USERNAME: &str = "SC-GitBot-Test";

#[actix_rt::test]
async fn test_parse_command_string_from_comment_line() {
    std::env::set_var(ENV_BOT_USERNAME, TEST_BOT_USERNAME);

    assert_eq!(
        parse_command_string_from_comment_line(&format!(
            "@{} this-is-a-command",
            std::env::var(ENV_BOT_USERNAME).expect("Empty bot username")
        )),
        Some("this-is-a-command")
    );

    assert_eq!(
        parse_command_string_from_comment_line("this-is-a-command"),
        None
    )
}

#[actix_rt::test]
async fn test_comment_action_from_comment() {
    assert_eq!(
        CommentAction::from_comment("noqa+"),
        Some(CommentAction::SkipQAStatus(true))
    );
    assert_eq!(
        CommentAction::from_comment("noqa-"),
        Some(CommentAction::SkipQAStatus(false))
    );
    assert_eq!(
        CommentAction::from_comment("qa+"),
        Some(CommentAction::QAStatus(true))
    );
    assert_eq!(
        CommentAction::from_comment("qa-"),
        Some(CommentAction::QAStatus(false))
    );
    assert_eq!(
        CommentAction::from_comment("automerge+"),
        Some(CommentAction::AutoMergeStatus(true))
    );
    assert_eq!(
        CommentAction::from_comment("automerge-"),
        Some(CommentAction::AutoMergeStatus(false))
    );
    assert_eq!(CommentAction::from_comment("this-is-a-command"), None);
}
