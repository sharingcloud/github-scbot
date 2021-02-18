//! Webhook logic tests

use github_scbot_conf::Config;
use github_scbot_database::{
    establish_single_test_connection,
    models::{
        AccountModel, PullRequestCreation, PullRequestModel, RepositoryCreation, RepositoryModel,
    },
    DbConn,
};

use super::test_config;
use crate::commands::{
    parse_command_string_from_comment_line, parse_commands, Command, CommandHandlingStatus,
};

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

#[actix_rt::test]
async fn test_command_rights() {
    let config = test_config();
    let conn = establish_single_test_connection(&config).unwrap();
    let (repo, mut pr) = arrange(&config, &conn);

    // PR creator should be authorized
    assert_eq!(
        parse_commands(
            &config,
            &conn,
            &repo,
            &mut pr,
            0,
            "me",
            "test-bot req+ @him",
        )
        .await
        .unwrap(),
        CommandHandlingStatus::Handled
    );

    // Someone else (non-admin) should be denied
    assert_eq!(
        parse_commands(
            &config,
            &conn,
            &repo,
            &mut pr,
            0,
            "him",
            "test-bot req+ @him",
        )
        .await
        .unwrap(),
        CommandHandlingStatus::Denied
    );

    // Ad admin should be authorized
    AccountModel::create(&conn, "admin", true).unwrap();
    assert_eq!(
        parse_commands(
            &config,
            &conn,
            &repo,
            &mut pr,
            0,
            "admin",
            "test-bot req+ @him",
        )
        .await
        .unwrap(),
        CommandHandlingStatus::Handled
    );
}

fn arrange(conf: &Config, conn: &DbConn) -> (RepositoryModel, PullRequestModel) {
    // Create a repository and a pull request
    let repo = RepositoryModel::create(
        &conn,
        RepositoryCreation {
            name: "TestRepo".into(),
            owner: "me".into(),
            ..RepositoryCreation::default(conf)
        },
    )
    .unwrap();
    let pr = PullRequestModel::create(
        &conn,
        PullRequestCreation {
            name: "PR 1".into(),
            number: 1,
            creator: "me".into(),
            ..PullRequestCreation::from_repository(&repo)
        },
    )
    .unwrap();

    (repo, pr)
}
