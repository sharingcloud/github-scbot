use github_scbot_conf::Config;
use github_scbot_types::{
    pulls::GHMergeStrategy,
    reviews::GHReviewState,
    status::{CheckStatus, QAStatus},
};

use super::import_export::{export_models_to_json, import_models_from_json};
use crate::{
    establish_single_test_connection,
    models::{
        AccountModel, ExternalAccountModel, ExternalAccountRightModel, MergeRuleCreation,
        MergeRuleModel, PullRequestCreation, PullRequestModel, RepositoryCreation, RepositoryModel,
        ReviewCreation, ReviewModel,
    },
};

fn test_init() {}

#[test]
fn create_repository() {
    test_init();

    let config = Config::from_env();
    let conn = establish_single_test_connection(&config).unwrap();
    let repo = RepositoryModel::create(
        &conn,
        RepositoryCreation {
            name: "TestRepo".into(),
            owner: "me".into(),
            ..RepositoryCreation::default(&config)
        },
    )
    .unwrap();

    assert_eq!(repo.id, 1);
    assert_eq!(repo.name, "TestRepo");
    assert_eq!(repo.owner, "me");
}

#[test]
fn list_repositories() {
    test_init();

    let config = Config::from_env();
    let conn = establish_single_test_connection(&config).unwrap();
    RepositoryModel::create(
        &conn,
        RepositoryCreation {
            name: "TestRepo".into(),
            owner: "me".into(),
            ..RepositoryCreation::default(&config)
        },
    )
    .unwrap();

    RepositoryModel::create(
        &conn,
        RepositoryCreation {
            name: "AnotherRepo".into(),
            owner: "me".into(),
            ..RepositoryCreation::default(&config)
        },
    )
    .unwrap();

    let repos = RepositoryModel::list(&conn).unwrap();
    assert_eq!(repos.len(), 2);
}

#[test]
fn create_pull_request() {
    test_init();

    let config = Config::from_env();
    let conn = establish_single_test_connection(&config).unwrap();
    let repo = RepositoryModel::create(
        &conn,
        RepositoryCreation {
            name: "TestRepo".to_string(),
            owner: "me".to_string(),
            ..RepositoryCreation::default(&config)
        },
    )
    .unwrap();

    let pr = PullRequestModel::create(
        &conn,
        PullRequestCreation {
            number: 1234,
            name: "Toto".to_string(),
            ..PullRequestCreation::from_repository(&repo)
        },
    )
    .unwrap();

    assert_eq!(pr.id, 1);
    assert_eq!(pr.repository_id, repo.id);
    assert_eq!(pr.get_number(), 1234);
}

#[test]
fn test_export_models_to_json() {
    test_init();

    let config = Config::from_env();
    let conn = establish_single_test_connection(&config).unwrap();
    let repo = RepositoryModel::create(
        &conn,
        RepositoryCreation {
            name: "TestRepo".into(),
            owner: "me".into(),
            ..RepositoryCreation::default(&config)
        },
    )
    .unwrap();

    let pr = PullRequestModel::create(
        &conn,
        PullRequestCreation {
            number: 1234,
            name: "Toto".into(),
            ..PullRequestCreation::from_repository(&repo)
        },
    )
    .unwrap();

    ReviewModel::create(
        &conn,
        ReviewCreation {
            pull_request_id: pr.id,
            required: true,
            state: GHReviewState::Commented.to_string(),
            username: "toto",
            valid: true,
        },
    )
    .unwrap();

    MergeRuleModel::create(
        &conn,
        MergeRuleCreation {
            repository_id: repo.id,
            base_branch: "base".into(),
            head_branch: "head".into(),
            strategy: GHMergeStrategy::Merge.to_string(),
        },
    )
    .unwrap();

    ExternalAccountModel::create(&conn, "ext", "pub", "pri").unwrap();

    let mut buffer = Vec::new();
    export_models_to_json(&conn, &mut buffer).unwrap();

    let buffer_string = String::from_utf8(buffer).unwrap();
    assert!(buffer_string.contains(r#""name": "TestRepo""#));
    assert!(buffer_string.contains(r#""number": 1234"#));
    assert!(buffer_string.contains(r#""username": "toto"#));
    assert!(buffer_string.contains(r#""username": "ext"#));
    assert!(buffer_string.contains(r#""strategy": "merge"#));
}

#[test]
fn test_import_models_from_json() {
    test_init();

    let config = Config::from_env();
    let conn = establish_single_test_connection(&config).unwrap();
    let repo = RepositoryModel::create(
        &conn,
        RepositoryCreation {
            name: "TestRepo".into(),
            owner: "me".into(),
            ..RepositoryCreation::default(&config)
        },
    )
    .unwrap();

    PullRequestModel::create(
        &conn,
        PullRequestCreation {
            number: 1234,
            name: "Toto".into(),
            ..PullRequestCreation::from_repository(&repo)
        },
    )
    .unwrap();

    let sample = serde_json::json!({
        "repositories": [
            {
                "id": 1,
                "name": "TestRepo",
                "owner": "me",
                "pr_title_validation_regex": "[a-z]*",
                "default_needed_reviewers_count": 2,
                "default_strategy": "merge"
            },
            {
                "id": 2,
                "name": "AnotherRepo",
                "owner": "me",
                "pr_title_validation_regex": "",
                "default_needed_reviewers_count": 3,
                "default_strategy": "merge"
            }
        ],
        "pull_requests": [
            {
                "id": 1,
                "repository_id": 1,
                "number": 1234,
                "name": "Tutu",
                "automerge": false,
                "step": "step/awaiting-review",
                "check_status": "waiting",
                "status_comment_id": 1,
                "qa_status": "waiting",
                "wip": false,
                "needed_reviewers_count": 2,
                "locked": false,
                "merged": false,
                "base_branch": "a",
                "head_branch": "b",
                "closed": false,
                "creator": "ghost"
            },
            {
                "id": 2,
                "repository_id": 1,
                "number": 1235,
                "name": "Tata",
                "automerge": true,
                "step": "step/wip",
                "check_status": "pass",
                "status_comment_id": 0,
                "qa_status": "pass",
                "wip": true,
                "needed_reviewers_count": 2,
                "locked": true,
                "merged": false,
                "base_branch": "a",
                "head_branch": "b",
                "closed": false,
                "creator": "me"
            }
        ],
        "reviews": [
            {
                "id": 1,
                "pull_request_id": 1,
                "username": "tutu",
                "state": "commented",
                "required": true,
                "valid": true
            }
        ],
        "merge_rules": [
            {
                "id": 1,
                "repository_id": 1,
                "base_branch": "base",
                "head_branch": "head",
                "strategy": "merge"
            }
        ],
        "accounts": [
            {
                "username": "ghost",
                "is_admin": false
            },
            {
                "username": "me",
                "is_admin": true
            }
        ],
        "external_accounts": [
            {
                "username": "ext",
                "public_key": "pub",
                "private_key": "priv"
            }
        ],
        "external_account_rights": [
            {
                "username": "ext",
                "repository_id": 1
            }
        ]
    });

    import_models_from_json(&config, &conn, sample.to_string().as_bytes()).unwrap();

    let rep_1 = RepositoryModel::get_from_owner_and_name(&conn, "me", "TestRepo").unwrap();
    let rep_2 = RepositoryModel::get_from_owner_and_name(&conn, "me", "AnotherRepo").unwrap();
    let pr_1 = PullRequestModel::get_from_repository_id_and_number(&conn, rep_1.id, 1234).unwrap();
    let pr_2 = PullRequestModel::get_from_repository_id_and_number(&conn, rep_1.id, 1235).unwrap();
    let review_1 = ReviewModel::get_from_pull_request_and_username(&conn, pr_1.id, "tutu").unwrap();
    let rule_1 = MergeRuleModel::get_from_branches(&conn, rep_1.id, "base", "head").unwrap();
    let acc_1 = AccountModel::get_from_username(&conn, "me").unwrap();
    let ext_acc_1 = ExternalAccountModel::get_from_username(&conn, "ext").unwrap();
    let ext_acc_right_1 = ExternalAccountRightModel::get_right(&conn, "ext", rep_1.id).unwrap();

    assert_eq!(rep_1.pr_title_validation_regex, "[a-z]*");
    assert_eq!(rep_2.pr_title_validation_regex, "");
    assert_eq!(pr_1.name, "Tutu");
    assert_eq!(pr_1.automerge, false);
    assert_eq!(pr_1.get_checks_status().unwrap(), CheckStatus::Waiting);
    assert_eq!(pr_1.get_qa_status().unwrap(), QAStatus::Waiting);
    assert_eq!(pr_2.name, "Tata");
    assert_eq!(pr_2.automerge, true);
    assert_eq!(pr_2.get_checks_status().unwrap(), CheckStatus::Pass);
    assert_eq!(pr_2.get_qa_status().unwrap(), QAStatus::Pass);
    assert_eq!(review_1.required, true);
    assert_eq!(acc_1.is_admin, true);
    assert_eq!(review_1.get_review_state(), GHReviewState::Commented);
    assert!(matches!(rule_1.get_strategy(), GHMergeStrategy::Merge));
    assert_eq!(ext_acc_1.public_key, "pub");
    assert_eq!(ext_acc_right_1.username, "ext");
}
