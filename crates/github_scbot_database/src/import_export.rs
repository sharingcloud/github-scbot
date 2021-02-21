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
        AccountModel, ExternalAccountModel, ExternalAccountRightModel, PullRequestCreation,
        PullRequestModel, RepositoryCreation, RepositoryModel, ReviewModel,
    },
    DbConn,
};
use crate::models::{MergeRuleCreation, MergeRuleModel};

/// Import error.
#[derive(Debug, Error)]
pub enum ImportError {
    /// Wraps [`serde_json::Error`].
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),

    /// Wraps [`std::io::Error`] with a path.
    #[error("IO error on file {0}.")]
    IOError(PathBuf, #[source] std::io::Error),

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
    IOError(PathBuf, #[source] std::io::Error),
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
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `writer` - Output stream
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
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `reader` - Input stream
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

        let repo = RepositoryModel::get_or_create(
            conn,
            RepositoryCreation {
                owner: repository.owner.clone(),
                name: repository.name.clone(),
                ..RepositoryCreation::default(config)
            },
        )?;
        repo_id_map.insert(repository.id, repo.id);
        repository.id = repo.id;
        repository.save(conn)?;

        repo_map.insert(repository.id, repository);
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
        let mr = MergeRuleModel::get_or_create(
            conn,
            MergeRuleCreation {
                repository_id: *repo_id,
                base_branch: merge_rule.base_branch.clone(),
                head_branch: merge_rule.head_branch.clone(),
                strategy: merge_rule.get_strategy().to_string(),
            },
        )?;
        merge_rule.id = mr.id;
        merge_rule.save(conn)?;
    }

    // Create or update pull requests
    for pull_request in &mut model.pull_requests {
        println!("> Importing pull request #{}", pull_request.get_number());

        let repo_id = repo_id_map
            .get(&pull_request.repository_id)
            .ok_or(ImportError::UnknownRepositoryId(pull_request.repository_id))?;
        let repo = repo_map.get(repo_id).unwrap();

        let pr = PullRequestModel::get_or_create(
            conn,
            &repo,
            PullRequestCreation {
                repository_id: *repo_id,
                number: pull_request.get_number() as i32,
                ..PullRequestCreation::from_repository(&repo)
            },
        )?;
        pr_id_map.insert(pull_request.id, pr.id);
        pull_request.id = pr.id;
        pull_request.save(conn)?;

        pr_map.insert(pull_request.id, pull_request);
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
        let repo = repo_map.get(&pr.id).unwrap();

        let rvw = ReviewModel::create_or_update(
            conn,
            &repo,
            &pr,
            &review.username,
            Some(review.get_review_state()),
            Some(review.required),
            Some(review.valid),
        )?;

        // Update pull request if needed
        review.id = rvw.id;
        review.save(conn)?;
    }

    for account in &mut model.accounts {
        println!("> Importing account '{}'", account.username);

        // Try to create account
        let _ = AccountModel::get_or_create(conn, &account.username, account.is_admin)?;

        // Update
        account.save(&conn)?;
    }

    // Create or update external accounts
    for account in &mut model.external_accounts {
        println!("> Importing external account '{}'", account.username);

        // Try to create account
        let _ = ExternalAccountModel::get_or_create(
            conn,
            &account.username,
            &account.public_key,
            &account.private_key,
        )?;

        // Update
        account.save(&conn)?;
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
