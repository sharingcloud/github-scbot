use std::{
    collections::HashMap,
    io::{Read, Write},
};

use github_scbot_domain_models::{
    Account, ExternalAccount, ExternalAccountRight, MergeRule, PullRequest, Repository,
    RequiredReviewer,
};
use serde::{Deserialize, Serialize};

use crate::{DatabaseError, DbService, Result};

#[derive(Debug, Serialize, Deserialize)]
struct ExchangeData {
    repositories: Vec<Repository>,
    pull_requests: Vec<PullRequest>,
    required_reviewers: Vec<RequiredReviewer>,
    merge_rules: Vec<MergeRule>,
    accounts: Vec<Account>,
    external_accounts: Vec<ExternalAccount>,
    external_account_rights: Vec<ExternalAccountRight>,
}

pub struct Exchanger;

impl Exchanger {
    pub async fn export_to_json<W: Write>(
        db_service: &mut dyn DbService,
        writer: &mut W,
    ) -> Result<()> {
        let data = ExchangeData {
            accounts: db_service.accounts_all().await?,
            external_account_rights: db_service.external_account_rights_all().await?,
            external_accounts: db_service.external_accounts_all().await?,
            merge_rules: db_service.merge_rules_all().await?,
            pull_requests: db_service.pull_requests_all().await?,
            repositories: db_service.repositories_all().await?,
            required_reviewers: db_service.required_reviewers_all().await?,
        };

        serde_json::to_writer_pretty(writer, &data)
            .map_err(|e| DatabaseError::ExchangeJsonError { source: e })?;

        Ok(())
    }

    pub async fn import_from_json<R: Read>(
        db_service: &mut dyn DbService,
        reader: R,
    ) -> Result<()> {
        let data: ExchangeData = serde_json::from_reader(reader)
            .map_err(|e| DatabaseError::ExchangeJsonError { source: e })?;

        let mut repo_id_map = HashMap::new();
        let mut pr_id_map = HashMap::new();
        let mut repo_map = HashMap::new();
        let mut pr_map = HashMap::new();

        for repository in data.repositories {
            println!(
                "> Importing repository '{}/{}'",
                repository.owner, repository.name
            );

            let repository_id = repository.id;
            let new_repository = Self::create_or_update_repository(db_service, repository).await?;
            repo_id_map.insert(repository_id, new_repository.id);
            repo_map.insert(new_repository.id, new_repository);
        }

        for mut merge_rule in data.merge_rules {
            let repo_id = repo_id_map.get(&merge_rule.repository_id).unwrap();
            let repo = repo_map.get(repo_id).unwrap();

            println!(
                "> Importing merge rule '{}' (base) <- '{}' (head), strategy '{}' for repository '{}/{}'",
                merge_rule.base_branch,
                merge_rule.head_branch,
                merge_rule.strategy,
                repo.owner,
                repo.name
            );

            merge_rule.repository_id = *repo_id;
            Self::create_or_update_merge_rule(db_service, repo, merge_rule).await?;
        }

        for mut pull_request in data.pull_requests {
            let repo_id = repo_id_map.get(&pull_request.repository_id).unwrap();
            let repo = repo_map.get(repo_id).unwrap();
            let pr_id = pull_request.id;

            println!(
                "> Importing pull request #{} for repository '{}/{}'",
                pull_request.number, repo.owner, repo.name
            );

            pull_request.repository_id = *repo_id;
            let new_pr =
                Self::create_or_update_pull_request(db_service, repo, pull_request).await?;

            pr_id_map.insert(pr_id, new_pr.id);
            pr_map.insert(new_pr.id, new_pr);
        }

        for account in data.accounts {
            println!("> Importing account '{}'", account.username);

            Self::create_or_update_account(db_service, account).await?;
        }

        for account in data.external_accounts {
            println!("> Importing external account '{}'", account.username);

            Self::create_or_update_external_account(db_service, account).await?;
        }

        for mut reviewer in data.required_reviewers {
            let pr_id = pr_id_map.get(&reviewer.pull_request_id).unwrap();
            let pr = pr_map.get(pr_id).unwrap();
            let repo = repo_map.get(&pr.repository_id).unwrap();

            println!(
                "> Importing required reviewer '{}' for PR '#{}' for repository '{}/{}'",
                reviewer.username, pr.number, repo.owner, repo.name
            );

            reviewer.pull_request_id = *pr_id;
            Self::create_or_update_reviewer(db_service, repo, pr, reviewer).await?;
        }

        for mut right in data.external_account_rights {
            let repo_id = repo_id_map.get(&right.repository_id).unwrap();
            let repo = repo_map.get(repo_id).unwrap();

            println!(
                "> Importing external account right for '{}' on repository '{}/{}'",
                right.username, repo.owner, repo.name
            );

            right.repository_id = *repo_id;
            Self::create_or_update_external_account_right(db_service, repo, right).await?;
        }

        Ok(())
    }

    async fn create_or_update_repository(
        db_service: &mut dyn DbService,
        repository: Repository,
    ) -> Result<Repository> {
        match db_service
            .repositories_get(&repository.owner, &repository.name)
            .await?
        {
            Some(_) => db_service.repositories_update(repository).await,
            None => db_service.repositories_create(repository).await,
        }
    }

    async fn create_or_update_pull_request(
        db_service: &mut dyn DbService,
        repository: &Repository,
        pull_request: PullRequest,
    ) -> Result<PullRequest> {
        match db_service
            .pull_requests_get(&repository.owner, &repository.name, pull_request.number)
            .await?
        {
            Some(_) => db_service.pull_requests_update(pull_request).await,
            None => db_service.pull_requests_create(pull_request).await,
        }
    }

    async fn create_or_update_merge_rule(
        db_service: &mut dyn DbService,
        repository: &Repository,
        merge_rule: MergeRule,
    ) -> Result<MergeRule> {
        match db_service
            .merge_rules_get(
                &repository.owner,
                &repository.name,
                merge_rule.base_branch.clone(),
                merge_rule.head_branch.clone(),
            )
            .await?
        {
            Some(_) => db_service.merge_rules_update(merge_rule).await,
            None => db_service.merge_rules_create(merge_rule).await,
        }
    }

    async fn create_or_update_account(
        db_service: &mut dyn DbService,
        account: Account,
    ) -> Result<Account> {
        match db_service.accounts_get(&account.username).await? {
            Some(_) => db_service.accounts_update(account).await,
            None => db_service.accounts_create(account).await,
        }
    }

    async fn create_or_update_external_account(
        db_service: &mut dyn DbService,
        exa: ExternalAccount,
    ) -> Result<ExternalAccount> {
        match db_service.external_accounts_get(&exa.username).await? {
            Some(_) => db_service.external_accounts_update(exa).await,
            None => db_service.external_accounts_create(exa).await,
        }
    }

    async fn create_or_update_reviewer(
        db_service: &mut dyn DbService,
        repository: &Repository,
        pull_request: &PullRequest,
        reviewer: RequiredReviewer,
    ) -> Result<RequiredReviewer> {
        match db_service
            .required_reviewers_get(
                &repository.owner,
                &repository.name,
                pull_request.number,
                &reviewer.username,
            )
            .await?
        {
            Some(s) => Ok(s),
            None => db_service.required_reviewers_create(reviewer).await,
        }
    }

    async fn create_or_update_external_account_right(
        db_service: &mut dyn DbService,
        repository: &Repository,
        right: ExternalAccountRight,
    ) -> Result<ExternalAccountRight> {
        match db_service
            .external_account_rights_get(&repository.owner, &repository.name, &right.username)
            .await?
        {
            Some(s) => Ok(s),
            None => db_service.external_account_rights_create(right).await,
        }
    }
}
