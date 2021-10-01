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
};
use crate::models::{IDatabaseAdapter, MergeRuleModel};

#[cfg(test)]
mod tests;

/// Import error.
#[derive(Debug, Error, Clone)]
pub enum ImportError {
    /// Serde error.
    #[error("Serde error: {0}")]
    SerdeError(String),

    /// IO error with a path.
    #[error("IO error on file {0} ({1}).")]
    IoError(PathBuf, String),

    /// Unknown repository ID.
    #[error("Unknown repository ID in file: {0}")]
    UnknownRepositoryId(i32),

    /// Unknown pull request ID.
    #[error("Unknown pull request ID in file: {0}")]
    UnknownPullRequestId(i32),
}

/// Export error.
#[derive(Debug, Error, Clone)]
pub enum ExportError {
    /// Serde error.
    #[error("Serde error: {0}")]
    SerdeError(String),

    /// IO error with a path.
    #[error("IO error on file {0} ({1}).")]
    IoError(PathBuf, String),
}

impl From<serde_json::Error> for ImportError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerdeError(err.to_string())
    }
}

impl From<serde_json::Error> for ExportError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerdeError(err.to_string())
    }
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
pub async fn export_models_to_json<W>(
    db_adapter: &dyn IDatabaseAdapter,
    writer: &mut W,
) -> Result<()>
where
    W: Write,
{
    let model = ImportExportModel {
        repositories: db_adapter.repository().list().await?,
        pull_requests: db_adapter.pull_request().list().await?,
        reviews: db_adapter.review().list().await?,
        merge_rules: db_adapter.merge_rule().list().await?,
        accounts: db_adapter.account().list().await?,
        external_accounts: db_adapter.external_account().list().await?,
        external_account_rights: db_adapter.external_account_right().list().await?,
    };

    serde_json::to_writer_pretty(writer, &model).map_err(ExportError::from)?;

    Ok(())
}

/// Import database models from JSON.
#[allow(clippy::missing_panics_doc)]
pub async fn import_models_from_json<R>(
    config: &Config,
    db_adapter: &dyn IDatabaseAdapter,
    reader: R,
) -> Result<()>
where
    R: Read,
{
    let mut model: ImportExportModel =
        serde_json::from_reader(reader).map_err(ImportError::from)?;

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

        let repo = RepositoryModel::builder_from_model(config, repository)
            .create_or_update(db_adapter.repository())
            .await?;

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

        MergeRuleModel::builder_from_model(repo, merge_rule)
            .create_or_update(db_adapter.merge_rule())
            .await?;
    }

    // Create or update pull requests
    for pull_request in &mut model.pull_requests {
        println!("> Importing pull request #{}", pull_request.number());

        let repo_id = repo_id_map
            .get(&pull_request.repository_id())
            .ok_or_else(|| ImportError::UnknownRepositoryId(pull_request.repository_id()))?;
        let repo = repo_map.get(repo_id).unwrap();

        let pr = PullRequestModel::builder_from_model(repo, pull_request)
            .create_or_update(db_adapter.pull_request())
            .await?;

        pr_id_map.insert(pull_request.id(), pr.id());
        pr_map.insert(pr.id(), pr);
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
        let repo = repo_map.get(&pr.repository_id()).unwrap();

        ReviewModel::builder_from_model(repo, pr, review)
            .create_or_update(db_adapter.review())
            .await?;
    }

    for account in &mut model.accounts {
        println!("> Importing account '{}'", account.username);

        // Try to create account
        AccountModel::builder_from_model(account)
            .create_or_update(db_adapter.account())
            .await?;
    }

    // Create or update external accounts
    for account in &mut model.external_accounts {
        println!("> Importing external account '{}'", account.username);

        // Try to create account
        ExternalAccountModel::builder_from_model(account)
            .create_or_update(db_adapter.external_account())
            .await?;
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

        db_adapter
            .external_account_right()
            .add_right(&right.username, repo)
            .await?;
    }

    Ok(())
}
