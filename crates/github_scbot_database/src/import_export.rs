//! Database import/export module.

use std::{
    collections::HashMap,
    io::{Read, Write},
    path::PathBuf,
};

use github_scbot_conf::Config;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{
    errors::Result,
    models::{
        AccountModel, ExternalAccountModel, ExternalAccountRightModel, PullRequestModel,
        RepositoryModel, ReviewModel,
    },
    DbConn,
};
use crate::models::MergeRuleModel;

/// Import error.
#[derive(Debug, Error)]
pub enum ImportError {
    /// Wraps [`serde_json::Error`].
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),

    /// Wraps [`std::io::Error`] with a path.
    #[error("IO error on file {0}.")]
    IoError(PathBuf, #[source] std::io::Error),

    /// Unknown repository ID.
    #[error("Unknown repository ID in file: {0}")]
    UnknownRepositoryId(i32),

    /// Unknown pull request ID.
    #[error("Unknown pull request ID in file: {0}")]
    UnknownPullRequestId(i32),
}

/// Export error.
#[derive(Debug, Error)]
pub enum ExportError {
    /// Wraps [`serde_json::Error`].
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),

    /// Wraps [`std::io::Error`] with a path.
    #[error("IO error on file {0}.")]
    IoError(PathBuf, #[source] std::io::Error),
}

#[derive(Debug, Deserialize, Serialize)]
struct ImportExportModel {
    repositories: Vec<RepositoryModel>,
    pull_requests: Vec<PullRequestModel>,
    reviews: Vec<ReviewModel>,
    merge_rules: Vec<MergeRuleModel>,
    accounts: Vec<AccountModel>,
    external_accounts: Vec<ExternalAccountModel>,
    external_account_rights: Vec<ExternalAccountRightModel>,
}

/// Export database models to JSON.
pub fn export_models_to_json<W>(conn: &DbConn, writer: &mut W) -> Result<()>
where
    W: Write,
{
    let model = ImportExportModel {
        repositories: RepositoryModel::list(conn)?,
        pull_requests: PullRequestModel::list(conn)?,
        reviews: ReviewModel::list(conn)?,
        merge_rules: MergeRuleModel::list(conn)?,
        accounts: AccountModel::list(conn)?,
        external_accounts: ExternalAccountModel::list(conn)?,
        external_account_rights: ExternalAccountRightModel::list(conn)?,
    };

    serde_json::to_writer_pretty(writer, &model).map_err(ExportError::SerdeError)?;

    Ok(())
}

/// Import database models from JSON.
pub fn import_models_from_json<R>(config: &Config, conn: &DbConn, reader: R) -> Result<()>
where
    R: Read,
{
    let mut model: ImportExportModel =
        serde_json::from_reader(reader).map_err(ImportError::SerdeError)?;

    let mut repo_id_map = HashMap::new();
    let mut repo_map = HashMap::new();
    let mut pr_id_map = HashMap::new();
    let mut pr_map = HashMap::new();

    // Create or update repositories
    for repository in &mut model.repositories {
        println!(
            "> Importing repository {}/{}",
            repository.owner, repository.name
        );

        let repo =
            RepositoryModel::builder_from_model(config, repository).create_or_update(conn)?;

        repo_id_map.insert(repository.id, repo.id);
        repo_map.insert(repo.id, repo);
    }

    // Create or update merge rules
    for merge_rule in &mut model.merge_rules {
        println!(
            "> Importing merge rule '{}' (base) <- '{}' (head), strategy '{}' for repository ID {}",
            merge_rule.base_branch,
            merge_rule.head_branch,
            merge_rule.get_strategy().to_string(),
            merge_rule.repository_id
        );

        let repo_id = repo_id_map
            .get(&merge_rule.repository_id)
            .ok_or(ImportError::UnknownRepositoryId(merge_rule.repository_id))?;
        let repo = repo_map.get(repo_id).unwrap();

        MergeRuleModel::builder_from_model(repo, merge_rule).create_or_update(conn)?;
    }

    // Create or update pull requests
    for pull_request in &mut model.pull_requests {
        println!("> Importing pull request #{}", pull_request.get_number());

        let repo_id = repo_id_map
            .get(&pull_request.repository_id)
            .ok_or(ImportError::UnknownRepositoryId(pull_request.repository_id))?;
        let repo = repo_map.get(repo_id).unwrap();

        let pr = PullRequestModel::builder_from_model(repo, pull_request).create_or_update(conn)?;

        pr_id_map.insert(pull_request.id, pr.id);
        pr_map.insert(pr.id, pr);
    }

    // Create or update reviews
    for review in &mut model.reviews {
        println!(
            "> Importing review for PR {} by @{}",
            review.id, review.username
        );

        let pr_id = pr_id_map
            .get(&review.pull_request_id)
            .ok_or(ImportError::UnknownPullRequestId(review.pull_request_id))?;
        let pr = pr_map.get(pr_id).unwrap();
        let repo = repo_map.get(&pr.repository_id).unwrap();

        ReviewModel::builder_from_model(&repo, &pr, review).create_or_update(conn)?;
    }

    for account in &mut model.accounts {
        println!("> Importing account '{}'", account.username);

        // Try to create account
        AccountModel::builder_from_model(account).create_or_update(conn)?;
    }

    // Create or update external accounts
    for account in &mut model.external_accounts {
        println!("> Importing external account '{}'", account.username);

        // Try to create account
        ExternalAccountModel::builder_from_model(account).create_or_update(conn)?;
    }

    // Create or update external account rights
    for right in &mut model.external_account_rights {
        println!(
            "> Importing external account right for '{}' on repository ID {}",
            right.username, right.repository_id
        );

        let repo_id = repo_id_map
            .get(&right.repository_id)
            .ok_or(ImportError::UnknownRepositoryId(right.repository_id))?;
        let repo = repo_map.get(repo_id).unwrap();
        ExternalAccountRightModel::add_right(conn, &right.username, &repo)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use github_scbot_types::{
        pulls::GhMergeStrategy,
        reviews::GhReviewState,
        status::{CheckStatus, QaStatus},
    };

    use super::*;
    use crate::{tests::using_test_db, DatabaseError};

    #[actix_rt::test]
    async fn test_export_models_to_json() -> Result<()> {
        let config = Config::from_env();

        using_test_db(
            &config.clone(),
            "test_db_export_models",
            |pool| async move {
                let conn = pool.get()?;

                let repo = RepositoryModel::builder(&config, "me", "TestRepo")
                    .create_or_update(&conn)
                    .unwrap();

                let pr = PullRequestModel::builder(&repo, 1234, "me")
                    .create_or_update(&conn)
                    .unwrap();

                ReviewModel::builder(&repo, &pr, "toto")
                    .state(GhReviewState::Commented)
                    .required(true)
                    .valid(true)
                    .create_or_update(&conn)
                    .unwrap();

                MergeRuleModel::builder(&repo, "base", "head")
                    .strategy(GhMergeStrategy::Merge)
                    .create_or_update(&conn)
                    .unwrap();

                ExternalAccountModel::builder("ext")
                    .public_key("pub")
                    .private_key("pri")
                    .create_or_update(&conn)
                    .unwrap();

                let mut buffer = Vec::new();
                export_models_to_json(&conn, &mut buffer).unwrap();

                let buffer_string = String::from_utf8(buffer).unwrap();
                assert!(buffer_string.contains(r#""name": "TestRepo""#));
                assert!(buffer_string.contains(r#""number": 1234"#));
                assert!(buffer_string.contains(r#""username": "toto"#));
                assert!(buffer_string.contains(r#""username": "ext"#));
                assert!(buffer_string.contains(r#""strategy": "merge"#));

                Ok::<_, DatabaseError>(())
            },
        )
        .await
    }

    #[actix_rt::test]
    async fn test_import_models_from_json() -> Result<()> {
        let config = Config::from_env();

        using_test_db(
            &config.clone(),
            "test_db_import_models",
            |pool| async move {
                let conn = pool.get()?;

                let repo = RepositoryModel::builder(&config, "me", "TestRepo")
                    .create_or_update(&conn)
                    .unwrap();

                PullRequestModel::builder(&repo, 1234, "me")
                    .name("Toto")
                    .create_or_update(&conn)
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
                            "manual_interaction": false
                        },
                        {
                            "id": 2,
                            "name": "AnotherRepo",
                            "owner": "me",
                            "pr_title_validation_regex": "",
                            "default_needed_reviewers_count": 3,
                            "default_strategy": "merge",
                            "manual_interaction": true
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

                let rep_1 =
                    RepositoryModel::get_from_owner_and_name(&conn, "me", "TestRepo").unwrap();
                let rep_2 =
                    RepositoryModel::get_from_owner_and_name(&conn, "me", "AnotherRepo").unwrap();
                let pr_1 =
                    PullRequestModel::get_from_repository_and_number(&conn, &rep_1, 1234).unwrap();
                let pr_2 =
                    PullRequestModel::get_from_repository_and_number(&conn, &rep_1, 1235).unwrap();
                let review_1 =
                    ReviewModel::get_from_pull_request_and_username(&conn, &rep_1, &pr_1, "tutu")
                        .unwrap();
                let rule_1 =
                    MergeRuleModel::get_from_branches(&conn, &rep_1, "base", "head").unwrap();
                let acc_1 = AccountModel::get_from_username(&conn, "me").unwrap();
                let ext_acc_1 = ExternalAccountModel::get_from_username(&conn, "ext").unwrap();
                let ext_acc_right_1 =
                    ExternalAccountRightModel::get_right(&conn, "ext", &rep_1).unwrap();

                assert_eq!(rep_1.pr_title_validation_regex, "[a-z]*");
                assert_eq!(rep_1.manual_interaction, false);
                assert_eq!(rep_2.pr_title_validation_regex, "");
                assert_eq!(rep_2.manual_interaction, true);
                assert_eq!(pr_1.name, "Tutu");
                assert_eq!(pr_1.automerge, false);
                assert_eq!(pr_1.get_checks_status(), CheckStatus::Waiting);
                assert_eq!(pr_1.get_qa_status(), QaStatus::Waiting);
                assert_eq!(pr_2.name, "Tata");
                assert_eq!(pr_2.automerge, true);
                assert_eq!(pr_2.get_checks_status(), CheckStatus::Pass);
                assert_eq!(pr_2.get_qa_status(), QaStatus::Pass);
                assert_eq!(review_1.required, true);
                assert_eq!(acc_1.is_admin, true);
                assert_eq!(review_1.get_review_state(), GhReviewState::Commented);
                assert!(matches!(rule_1.get_strategy(), GhMergeStrategy::Merge));
                assert_eq!(ext_acc_1.public_key, "pub");
                assert_eq!(ext_acc_right_1.username, "ext");

                Ok::<_, DatabaseError>(())
            },
        )
        .await
    }
}
