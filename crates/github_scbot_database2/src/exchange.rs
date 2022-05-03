use std::{
    collections::HashMap,
    io::{Read, Write},
};

use serde::{Deserialize, Serialize};

use crate::{
    Account, DatabaseError, DbService, ExternalAccount, ExternalAccountRight, MergeRule,
    PullRequest, Repository, RepositoryDB, RequiredReviewer,
};
use crate::{
    AccountDB, ExternalAccountDB, ExternalAccountRightDB, MergeRuleDB, PullRequestDB,
    RequiredReviewerDB, Result,
};

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
            accounts: db_service.accounts().all().await?,
            external_account_rights: db_service.external_account_rights().all().await?,
            external_accounts: db_service.external_accounts().all().await?,
            merge_rules: db_service.merge_rules().all().await?,
            pull_requests: db_service.pull_requests().all().await?,
            repositories: db_service.repositories().all().await?,
            required_reviewers: db_service.required_reviewers().all().await?,
        };

        serde_json::to_writer_pretty(writer, &data)
            .map_err(|e| DatabaseError::ExchangeError(e.into()))?;

        Ok(())
    }

    pub async fn import_from_json<R: Read>(
        db_service: &mut dyn DbService,
        reader: R,
    ) -> Result<()> {
        let data: ExchangeData =
            serde_json::from_reader(reader).map_err(|e| DatabaseError::ExchangeError(e.into()))?;

        let mut repo_id_map = HashMap::new();
        let mut pr_id_map = HashMap::new();
        let mut repo_map = HashMap::new();
        let mut pr_map = HashMap::new();

        for repository in data.repositories {
            println!(
                "> Importing repository '{}/{}'",
                repository.owner(),
                repository.name()
            );

            let repository_id = repository.id();
            let new_repository =
                Self::create_or_update_repository(&mut *db_service.repositories(), repository)
                    .await?;
            repo_id_map.insert(repository_id, new_repository.id());
            repo_map.insert(new_repository.id(), new_repository);
        }

        for mut merge_rule in data.merge_rules {
            let repo_id = repo_id_map.get(&merge_rule.repository_id()).unwrap();
            let repo = repo_map.get(repo_id).unwrap();

            println!(
                "> Importing merge rule '{}' (base) <- '{}' (head), strategy '{}' for repository '{}/{}'",
                merge_rule.base_branch(),
                merge_rule.head_branch(),
                merge_rule.strategy(),
                repo.owner(),
                repo.name()
            );

            merge_rule.set_repository_id(*repo_id);
            Self::create_or_update_merge_rule(&mut *db_service.merge_rules(), repo, merge_rule)
                .await?;
        }

        for mut pull_request in data.pull_requests {
            let repo_id = repo_id_map.get(&pull_request.repository_id()).unwrap();
            let repo = repo_map.get(repo_id).unwrap();
            let pr_id = pull_request.id();

            println!(
                "> Importing pull request #{} for repository '{}/{}'",
                pull_request.number(),
                repo.owner(),
                repo.name()
            );

            pull_request.set_repository_id(*repo_id);
            let new_pr = Self::create_or_update_pull_request(
                &mut *db_service.pull_requests(),
                repo,
                pull_request,
            )
            .await?;

            pr_id_map.insert(pr_id, new_pr.id());
            pr_map.insert(new_pr.id(), new_pr);
        }

        for account in data.accounts {
            println!("> Importing account '{}'", account.username());

            Self::create_or_update_account(&mut *db_service.accounts(), account).await?;
        }

        for account in data.external_accounts {
            println!("> Importing external account '{}'", account.username());

            Self::create_or_update_external_account(&mut *db_service.external_accounts(), account)
                .await?;
        }

        for mut reviewer in data.required_reviewers {
            let pr_id = pr_id_map.get(&reviewer.pull_request_id()).unwrap();
            let pr = pr_map.get(pr_id).unwrap();
            let repo = repo_map.get(&pr.repository_id()).unwrap();

            println!(
                "> Importing required reviewer '{}' for PR '#{}' for repository '{}/{}'",
                reviewer.username(),
                pr.number(),
                repo.owner(),
                repo.name()
            );

            reviewer.set_pull_request_id(*pr_id);
            Self::create_or_update_reviewer(
                &mut *db_service.required_reviewers(),
                repo,
                pr,
                reviewer,
            )
            .await?;
        }

        for mut right in data.external_account_rights {
            let repo_id = repo_id_map.get(&right.repository_id()).unwrap();
            let repo = repo_map.get(repo_id).unwrap();

            println!(
                "> Importing external account right for '{}' on repository '{}/{}'",
                right.username(),
                repo.owner(),
                repo.name()
            );

            right.set_repository_id(*repo_id);
            Self::create_or_update_external_account_right(
                &mut *db_service.external_account_rights(),
                repo,
                right,
            )
            .await?;
        }

        Ok(())
    }

    async fn create_or_update_repository(
        repo_db: &mut dyn RepositoryDB,
        repository: Repository,
    ) -> Result<Repository> {
        match repo_db.get(repository.owner(), repository.name()).await? {
            Some(_) => repo_db.update(repository).await,
            None => repo_db.create(repository).await,
        }
    }

    async fn create_or_update_pull_request(
        pr_db: &mut dyn PullRequestDB,
        repository: &Repository,
        pull_request: PullRequest,
    ) -> Result<PullRequest> {
        match pr_db
            .get(repository.owner(), repository.name(), pull_request.number())
            .await?
        {
            Some(_) => pr_db.update(pull_request).await,
            None => pr_db.create(pull_request).await,
        }
    }

    async fn create_or_update_merge_rule(
        mr_db: &mut dyn MergeRuleDB,
        repository: &Repository,
        merge_rule: MergeRule,
    ) -> Result<MergeRule> {
        match mr_db
            .get(
                repository.owner(),
                repository.name(),
                merge_rule.base_branch().clone(),
                merge_rule.head_branch().clone(),
            )
            .await?
        {
            Some(_) => mr_db.update(merge_rule).await,
            None => mr_db.create(merge_rule).await,
        }
    }

    async fn create_or_update_account(
        account_db: &mut dyn AccountDB,
        account: Account,
    ) -> Result<Account> {
        match account_db.get(account.username()).await? {
            Some(_) => account_db.update(account).await,
            None => account_db.create(account).await,
        }
    }

    async fn create_or_update_external_account(
        exa_db: &mut dyn ExternalAccountDB,
        exa: ExternalAccount,
    ) -> Result<ExternalAccount> {
        match exa_db.get(exa.username()).await? {
            Some(_) => exa_db.update(exa).await,
            None => exa_db.create(exa).await,
        }
    }

    async fn create_or_update_reviewer(
        reviewer_db: &mut dyn RequiredReviewerDB,
        repository: &Repository,
        pull_request: &PullRequest,
        reviewer: RequiredReviewer,
    ) -> Result<RequiredReviewer> {
        match reviewer_db
            .get(
                repository.owner(),
                repository.name(),
                pull_request.number(),
                reviewer.username(),
            )
            .await?
        {
            Some(s) => Ok(s),
            None => reviewer_db.create(reviewer).await,
        }
    }

    async fn create_or_update_external_account_right(
        right_db: &mut dyn ExternalAccountRightDB,
        repository: &Repository,
        right: ExternalAccountRight,
    ) -> Result<ExternalAccountRight> {
        match right_db
            .get(repository.owner(), repository.name(), right.username())
            .await?
        {
            Some(s) => Ok(s),
            None => right_db.create(right).await,
        }
    }
}
