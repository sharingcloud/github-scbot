use github_scbot_types::{
    pulls::GhMergeStrategy,
    reviews::GhReviewState,
    status::{CheckStatus, QaStatus},
};

use super::*;
use crate::{models::DatabaseAdapter, tests::using_test_db, DatabaseError};

#[actix_rt::test]
async fn test_export_models_to_json() -> Result<()> {
    using_test_db("test_db_export_models", |config, pool| async move {
        let db_adapter = DatabaseAdapter::new(pool);

        let repo = RepositoryModel::builder(&config, "me", "TestRepo")
            .create_or_update(db_adapter.repository())
            .await
            .unwrap();

        let pr = PullRequestModel::builder(&repo, 1234, "me")
            .create_or_update(db_adapter.pull_request())
            .await
            .unwrap();

        ReviewModel::builder(&repo, &pr, "toto")
            .state(GhReviewState::Commented)
            .required(true)
            .valid(true)
            .create_or_update(db_adapter.review())
            .await
            .unwrap();

        MergeRuleModel::builder(&repo, "base", "head")
            .strategy(GhMergeStrategy::Merge)
            .create_or_update(db_adapter.merge_rule())
            .await
            .unwrap();

        ExternalAccountModel::builder("ext")
            .public_key("pub")
            .private_key("pri")
            .create_or_update(db_adapter.external_account())
            .await
            .unwrap();

        let mut buffer = Vec::new();
        export_models_to_json(&db_adapter, &mut buffer)
            .await
            .unwrap();

        let buffer_string = String::from_utf8(buffer).unwrap();
        assert!(buffer_string.contains(r#""name": "TestRepo""#));
        assert!(buffer_string.contains(r#""number": 1234"#));
        assert!(buffer_string.contains(r#""username": "toto"#));
        assert!(buffer_string.contains(r#""username": "ext"#));
        assert!(buffer_string.contains(r#""strategy": "merge"#));

        Ok::<_, DatabaseError>(())
    })
    .await
}

#[actix_rt::test]
#[allow(clippy::too_many_lines)]
async fn test_import_models_from_json() -> Result<()> {
    using_test_db("test_db_import_models", |config, pool| async move {
        let db_adapter = DatabaseAdapter::new(pool);

        let repo = RepositoryModel::builder(&config, "me", "TestRepo")
            .create_or_update(db_adapter.repository())
            .await
            .unwrap();

        PullRequestModel::builder(&repo, 1234, "me")
            .name("Toto")
            .create_or_update(db_adapter.pull_request())
            .await
            .unwrap();

        let sample = serde_json::json!({
            "repositories": [
                {
                    "id": 1,
                    "name": "TestRepo",
                    "owner": "me",
                    "pr_title_validation_regex": "[a-z]*",
                    "default_needed_reviewers_count": 2,
                    "default_strategy": "merge",
                    "manual_interaction": false,
                    "default_automerge": false,
                    "default_enable_qa": true,
                    "default_enable_checks": true
                },
                {
                    "id": 2,
                    "name": "AnotherRepo",
                    "owner": "me",
                    "pr_title_validation_regex": "",
                    "default_needed_reviewers_count": 3,
                    "default_strategy": "merge",
                    "manual_interaction": true,
                    "default_automerge": true,
                    "default_enable_qa": true,
                    "default_enable_checks": false
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

        import_models_from_json(&config, &db_adapter, sample.to_string().as_bytes())
            .await
            .unwrap();

        let rep_1 = db_adapter
            .repository()
            .get_from_owner_and_name("me", "TestRepo")
            .await
            .unwrap();
        let rep_2 = db_adapter
            .repository()
            .get_from_owner_and_name("me", "AnotherRepo")
            .await
            .unwrap();
        let pr_1 = db_adapter
            .pull_request()
            .get_from_repository_and_number(&rep_1, 1234)
            .await
            .unwrap();
        let pr_2 = db_adapter
            .pull_request()
            .get_from_repository_and_number(&rep_1, 1235)
            .await
            .unwrap();
        let review_1 = db_adapter
            .review()
            .get_from_pull_request_and_username(&rep_1, &pr_1, "tutu")
            .await
            .unwrap();
        let rule_1 =
            MergeRuleModel::get_from_branches(db_adapter.merge_rule(), &rep_1, "base", "head")
                .await
                .unwrap();
        let acc_1 = db_adapter.account().get_from_username("me").await.unwrap();
        let ext_acc_1 = db_adapter
            .external_account()
            .get_from_username("ext")
            .await
            .unwrap();
        let ext_acc_right_1 = db_adapter
            .external_account_right()
            .get_right("ext", &rep_1)
            .await
            .unwrap();

        assert_eq!(rep_1.pr_title_validation_regex, "[a-z]*");
        assert!(!rep_1.manual_interaction);
        assert!(!rep_1.default_automerge);
        assert_eq!(rep_2.pr_title_validation_regex, "");
        assert!(rep_2.manual_interaction);
        assert!(rep_2.default_automerge);
        assert_eq!(pr_1.name(), "Tutu");
        assert!(!pr_1.automerge());
        assert_eq!(pr_1.check_status(), CheckStatus::Waiting);
        assert_eq!(pr_1.qa_status(), QaStatus::Waiting);
        assert_eq!(pr_2.name(), "Tata");
        assert!(pr_2.automerge());
        assert_eq!(pr_2.check_status(), CheckStatus::Pass);
        assert_eq!(pr_2.qa_status(), QaStatus::Pass);
        assert!(review_1.required());
        assert!(acc_1.is_admin);
        assert_eq!(review_1.get_review_state(), GhReviewState::Commented);
        assert!(matches!(rule_1.get_strategy(), GhMergeStrategy::Merge));
        assert_eq!(ext_acc_1.public_key, "pub");
        assert_eq!(ext_acc_right_1.username, "ext");

        Ok::<_, DatabaseError>(())
    })
    .await
}
