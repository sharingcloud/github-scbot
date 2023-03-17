use std::collections::HashMap;

use async_trait::async_trait;
use github_scbot_core::types::{pulls::GhMergeStrategy, rule_branch::RuleBranch, status::QaStatus};
use github_scbot_database_interface::{DbService, Result};
use github_scbot_domain_models::{
    Account, ExternalAccount, ExternalAccountRight, MergeRule, PullRequest, Repository,
    RequiredReviewer,
};

#[derive(Debug, Default)]
pub struct MemoryDb {
    repositories: HashMap<u64, Repository>,
    pull_requests: HashMap<u64, PullRequest>,
    accounts: HashMap<String, Account>,
    external_accounts: HashMap<String, ExternalAccount>,
    external_account_rights: HashMap<(String, u64), ExternalAccountRight>,
    merge_rules: HashMap<(u64, RuleBranch, RuleBranch), MergeRule>,
    required_reviewers: HashMap<(String, u64), RequiredReviewer>,
}

impl MemoryDb {
    pub fn new() -> Self {
        Default::default()
    }

    fn get_last_repository_id(&self) -> u64 {
        self.repositories.keys().max().copied().unwrap_or(0) + 1
    }

    fn get_last_pull_request_id(&self) -> u64 {
        self.pull_requests.keys().max().copied().unwrap_or(0) + 1
    }
}

#[async_trait]
impl DbService for MemoryDb {
    async fn accounts_create(&mut self, instance: Account) -> Result<Account> {
        self.accounts
            .insert(instance.username.clone(), instance.clone());
        Ok(instance)
    }

    async fn accounts_update(&mut self, instance: Account) -> Result<Account> {
        self.accounts
            .insert(instance.username.clone(), instance.clone());
        Ok(instance)
    }

    async fn accounts_all(&mut self) -> Result<Vec<Account>> {
        let mut values: Vec<_> = self.accounts.values().cloned().collect();
        values.sort_by(|a, b| a.username.cmp(&b.username));
        Ok(values)
    }

    async fn accounts_get(&mut self, username: &str) -> Result<Option<Account>> {
        Ok(self.accounts.get(username).cloned())
    }

    async fn accounts_delete(&mut self, username: &str) -> Result<bool> {
        if self.accounts.get(username).is_some() {
            self.accounts.remove(username);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn accounts_list_admins(&mut self) -> Result<Vec<Account>> {
        let mut values: Vec<_> = self
            .accounts
            .values()
            .filter(|a| a.is_admin)
            .cloned()
            .collect();
        values.sort_by(|a, b| a.username.cmp(&b.username));
        Ok(values)
    }

    async fn accounts_set_is_admin(&mut self, username: &str, value: bool) -> Result<Account> {
        let mut account = self.accounts_get_expect(username).await?;
        account.is_admin = value;
        self.accounts
            .insert(account.username.clone(), account.clone());
        Ok(account)
    }

    //////////////////////////
    // External account rights

    async fn external_account_rights_create(
        &mut self,
        instance: ExternalAccountRight,
    ) -> Result<ExternalAccountRight> {
        self.repositories_get_from_id_expect(instance.repository_id)
            .await?;
        self.external_accounts_get_expect(&instance.username)
            .await?;
        self.external_account_rights.insert(
            (instance.username.clone(), instance.repository_id),
            instance.clone(),
        );
        Ok(instance)
    }

    async fn external_account_rights_get(
        &mut self,
        owner: &str,
        name: &str,
        username: &str,
    ) -> Result<Option<ExternalAccountRight>> {
        if let Some(repo) = self.repositories_get(owner, name).await? {
            Ok(self
                .external_account_rights
                .get(&(username.to_owned(), repo.id))
                .cloned())
        } else {
            Ok(None)
        }
    }

    async fn external_account_rights_delete(
        &mut self,
        owner: &str,
        name: &str,
        username: &str,
    ) -> Result<bool> {
        if let Some(r) = self
            .external_account_rights_get(owner, name, username)
            .await?
        {
            self.external_account_rights
                .remove(&(username.to_owned(), r.repository_id));
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn external_account_rights_delete_all(&mut self, username: &str) -> Result<bool> {
        let keys_to_remove: Vec<_> = self
            .external_account_rights
            .keys()
            .filter(|(u, _)| u == username)
            .cloned()
            .collect();
        let has_values = !keys_to_remove.is_empty();
        for key in keys_to_remove {
            self.external_account_rights.remove(&key);
        }
        Ok(has_values)
    }

    async fn external_account_rights_list(
        &mut self,
        username: &str,
    ) -> Result<Vec<ExternalAccountRight>> {
        let mut values: Vec<_> = self
            .external_account_rights
            .iter()
            .filter(|((u, _), _)| u == username)
            .map(|(_, v)| v.clone())
            .collect();
        values.sort_by(|a, b| (&a.username, a.repository_id).cmp(&(&b.username, b.repository_id)));
        Ok(values)
    }

    async fn external_account_rights_all(&mut self) -> Result<Vec<ExternalAccountRight>> {
        let mut values: Vec<_> = self.external_account_rights.values().cloned().collect();
        values.sort_by(|a, b| (&a.username, a.repository_id).cmp(&(&b.username, b.repository_id)));
        Ok(values)
    }

    ////////////////////
    // External accounts

    async fn external_accounts_create(
        &mut self,
        instance: ExternalAccount,
    ) -> Result<ExternalAccount> {
        self.external_accounts
            .insert(instance.username.clone(), instance.clone());
        Ok(instance)
    }

    async fn external_accounts_update(
        &mut self,
        instance: ExternalAccount,
    ) -> Result<ExternalAccount> {
        self.external_accounts_get_expect(&instance.username)
            .await?;

        self.external_accounts
            .insert(instance.username.clone(), instance.clone());
        Ok(instance)
    }

    async fn external_accounts_get(&mut self, username: &str) -> Result<Option<ExternalAccount>> {
        Ok(self.external_accounts.get(username).cloned())
    }

    async fn external_accounts_delete(&mut self, username: &str) -> Result<bool> {
        if self.external_accounts.get(username).is_some() {
            // Cascade!
            let exrs_to_remove: Vec<_> = self
                .external_account_rights
                .values()
                .filter(|exr| exr.username == username)
                .cloned()
                .collect();
            for exr in exrs_to_remove {
                self.external_account_rights_delete_all(&exr.username)
                    .await?;
            }

            self.external_accounts.remove(username);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn external_accounts_all(&mut self) -> Result<Vec<ExternalAccount>> {
        let mut values: Vec<_> = self.external_accounts.values().cloned().collect();
        values.sort_by(|a, b| a.username.cmp(&b.username));
        Ok(values)
    }

    async fn external_accounts_set_keys(
        &mut self,
        username: &str,
        public_key: &str,
        private_key: &str,
    ) -> Result<ExternalAccount> {
        let mut account = self.external_accounts_get_expect(username).await?;
        account.public_key = public_key.to_owned();
        account.private_key = private_key.to_owned();
        self.external_accounts
            .insert(account.username.clone(), account.clone());
        Ok(account)
    }

    ///////////////
    // Health check

    async fn health_check(&mut self) -> Result<()> {
        Ok(())
    }

    //////////////
    // Merge rules

    async fn merge_rules_create(&mut self, instance: MergeRule) -> Result<MergeRule> {
        self.repositories_get_from_id_expect(instance.repository_id)
            .await?;

        self.merge_rules.insert(
            (
                instance.repository_id,
                instance.base_branch.clone(),
                instance.head_branch.clone(),
            ),
            instance.clone(),
        );
        Ok(instance)
    }

    async fn merge_rules_update(&mut self, instance: MergeRule) -> Result<MergeRule> {
        let repo = self
            .repositories_get_from_id_expect(instance.repository_id)
            .await?;
        self.merge_rules_get_expect(
            &repo.owner,
            &repo.name,
            instance.base_branch.clone(),
            instance.head_branch.clone(),
        )
        .await?;

        self.merge_rules.insert(
            (
                instance.repository_id,
                instance.base_branch.clone(),
                instance.head_branch.clone(),
            ),
            instance.clone(),
        );
        Ok(instance)
    }

    async fn merge_rules_get(
        &mut self,
        owner: &str,
        name: &str,
        base_branch: RuleBranch,
        head_branch: RuleBranch,
    ) -> Result<Option<MergeRule>> {
        if let Some(repo) = self.repositories_get(owner, name).await? {
            Ok(self
                .merge_rules
                .get(&(repo.id, base_branch, head_branch))
                .cloned())
        } else {
            Ok(None)
        }
    }

    async fn merge_rules_delete(
        &mut self,
        owner: &str,
        name: &str,
        base_branch: RuleBranch,
        head_branch: RuleBranch,
    ) -> Result<bool> {
        if let Some(r) = self
            .merge_rules_get(owner, name, base_branch, head_branch)
            .await?
        {
            self.merge_rules
                .remove(&(r.repository_id, r.base_branch, r.head_branch));
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn merge_rules_all(&mut self) -> Result<Vec<MergeRule>> {
        let mut values: Vec<_> = self.merge_rules.values().cloned().collect();
        values.sort_by(|a, b| {
            (a.repository_id, &a.base_branch, &a.head_branch).cmp(&(
                b.repository_id,
                &b.base_branch,
                &b.head_branch,
            ))
        });
        Ok(values)
    }

    async fn merge_rules_list(&mut self, owner: &str, name: &str) -> Result<Vec<MergeRule>> {
        if let Some(repo) = self.repositories_get(owner, name).await? {
            let mut values: Vec<_> = self
                .merge_rules
                .values()
                .filter(|r| r.repository_id == repo.id)
                .cloned()
                .collect();
            values.sort_by(|a, b| {
                (a.repository_id, &a.base_branch, &a.head_branch).cmp(&(
                    b.repository_id,
                    &b.base_branch,
                    &b.head_branch,
                ))
            });

            Ok(values)
        } else {
            Ok(vec![])
        }
    }

    ////////////////
    // Pull requests

    async fn pull_requests_create(&mut self, mut instance: PullRequest) -> Result<PullRequest> {
        self.repositories_get_from_id_expect(instance.repository_id)
            .await?;

        let id = self.get_last_pull_request_id();
        instance.id = id;
        self.pull_requests.insert(instance.id, instance.clone());
        Ok(instance)
    }

    async fn pull_requests_update(&mut self, instance: PullRequest) -> Result<PullRequest> {
        assert!(instance.id != 0);
        let repo = self
            .repositories_get_from_id_expect(instance.repository_id)
            .await?;
        self.pull_requests_get_expect(&repo.owner, &repo.name, instance.id)
            .await?;

        self.pull_requests.insert(instance.id, instance.clone());
        Ok(instance)
    }

    async fn pull_requests_get(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
    ) -> Result<Option<PullRequest>> {
        if let Some(repo) = self.repositories_get(owner, name).await? {
            Ok(self
                .pull_requests
                .values()
                .find(|pr| pr.repository_id == repo.id && pr.number == number)
                .cloned())
        } else {
            Ok(None)
        }
    }

    async fn pull_requests_get_from_id(&mut self, id: u64) -> Result<Option<PullRequest>> {
        Ok(self.pull_requests.get(&id).cloned())
    }

    async fn pull_requests_delete(&mut self, owner: &str, name: &str, number: u64) -> Result<bool> {
        if let Some(pr) = self.pull_requests_get(owner, name, number).await? {
            // Cascade!
            let reviewers_to_remove: Vec<_> = self
                .required_reviewers
                .values()
                .filter(|r| r.pull_request_id == pr.id)
                .cloned()
                .collect();
            for r in reviewers_to_remove {
                self.required_reviewers_delete(owner, name, number, &r.username)
                    .await?;
            }

            self.pull_requests.remove(&pr.id);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn pull_requests_list(&mut self, owner: &str, name: &str) -> Result<Vec<PullRequest>> {
        if let Some(repo) = self.repositories_get(owner, name).await? {
            let mut values: Vec<_> = self
                .pull_requests
                .values()
                .filter(|pr| pr.repository_id == repo.id)
                .cloned()
                .collect();
            values.sort_by(|a, b| (a.repository_id, a.number).cmp(&(b.repository_id, b.number)));
            Ok(values)
        } else {
            Ok(vec![])
        }
    }

    async fn pull_requests_all(&mut self) -> Result<Vec<PullRequest>> {
        let mut values: Vec<_> = self.pull_requests.values().cloned().collect();
        values.sort_by(|a, b| (a.repository_id, a.number).cmp(&(b.repository_id, b.number)));
        Ok(values)
    }

    async fn pull_requests_set_qa_status(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        status: QaStatus,
    ) -> Result<PullRequest> {
        self.repositories_get_expect(owner, name).await?;
        let mut pr = self.pull_requests_get_expect(owner, name, number).await?;
        pr.qa_status = status;
        self.pull_requests.insert(pr.id, pr.clone());
        Ok(pr)
    }

    async fn pull_requests_set_needed_reviewers_count(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        count: u64,
    ) -> Result<PullRequest> {
        self.repositories_get_expect(owner, name).await?;
        let mut pr = self.pull_requests_get_expect(owner, name, number).await?;
        pr.needed_reviewers_count = count;
        self.pull_requests.insert(pr.id, pr.clone());
        Ok(pr)
    }

    async fn pull_requests_set_status_comment_id(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        id: u64,
    ) -> Result<PullRequest> {
        self.repositories_get_expect(owner, name).await?;
        let mut pr = self.pull_requests_get_expect(owner, name, number).await?;
        pr.status_comment_id = id;
        self.pull_requests.insert(pr.id, pr.clone());
        Ok(pr)
    }

    async fn pull_requests_set_checks_enabled(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        value: bool,
    ) -> Result<PullRequest> {
        self.repositories_get_expect(owner, name).await?;
        let mut pr = self.pull_requests_get_expect(owner, name, number).await?;
        pr.checks_enabled = value;
        self.pull_requests.insert(pr.id, pr.clone());
        Ok(pr)
    }

    async fn pull_requests_set_automerge(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        value: bool,
    ) -> Result<PullRequest> {
        self.repositories_get_expect(owner, name).await?;
        let mut pr = self.pull_requests_get_expect(owner, name, number).await?;
        pr.automerge = value;
        self.pull_requests.insert(pr.id, pr.clone());
        Ok(pr)
    }

    async fn pull_requests_set_locked(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        value: bool,
    ) -> Result<PullRequest> {
        self.repositories_get_expect(owner, name).await?;
        let mut pr = self.pull_requests_get_expect(owner, name, number).await?;
        pr.locked = value;
        self.pull_requests.insert(pr.id, pr.clone());
        Ok(pr)
    }

    async fn pull_requests_set_strategy_override(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        strategy: Option<GhMergeStrategy>,
    ) -> Result<PullRequest> {
        self.repositories_get_expect(owner, name).await?;
        let mut pr = self.pull_requests_get_expect(owner, name, number).await?;
        pr.strategy_override = strategy;
        self.pull_requests.insert(pr.id, pr.clone());
        Ok(pr)
    }

    ///////////////
    // Repositories

    async fn repositories_create(&mut self, mut instance: Repository) -> Result<Repository> {
        let id = self.get_last_repository_id();
        instance.id = id;
        self.repositories.insert(instance.id, instance.clone());
        Ok(instance)
    }

    async fn repositories_update(&mut self, instance: Repository) -> Result<Repository> {
        assert!(instance.id != 0);
        self.repositories.insert(instance.id, instance.clone());
        Ok(instance)
    }

    async fn repositories_all(&mut self) -> Result<Vec<Repository>> {
        let mut values: Vec<_> = self.repositories.values().cloned().collect();
        values.sort_by(|a, b| (&a.owner, &a.name).cmp(&(&b.owner, &b.name)));

        Ok(values)
    }

    async fn repositories_get(&mut self, owner: &str, name: &str) -> Result<Option<Repository>> {
        Ok(self
            .repositories
            .values()
            .find(|v| v.owner == owner && v.name == name)
            .cloned())
    }

    async fn repositories_get_from_id(&mut self, id: u64) -> Result<Option<Repository>> {
        Ok(self.repositories.get(&id).cloned())
    }

    async fn repositories_delete(&mut self, owner: &str, name: &str) -> Result<bool> {
        if let Some(v) = self.repositories_get(owner, name).await? {
            // Cascades!
            let rights_to_remove: Vec<_> = self
                .external_account_rights
                .keys()
                .filter(|(_, i)| *i == v.id)
                .cloned()
                .collect();
            for (username, _) in rights_to_remove {
                self.external_account_rights_delete(owner, name, &username)
                    .await?;
            }
            let pull_requests_to_remove: Vec<_> = self
                .pull_requests
                .values()
                .filter(|pr| pr.repository_id == v.id)
                .cloned()
                .collect();
            for pr in pull_requests_to_remove {
                self.pull_requests_delete(owner, name, pr.number).await?;
            }
            let merge_rules_to_remove: Vec<_> = self
                .merge_rules
                .values()
                .filter(|mr| mr.repository_id == v.id)
                .cloned()
                .collect();
            for mr in merge_rules_to_remove {
                self.merge_rules_delete(owner, name, mr.base_branch, mr.head_branch)
                    .await?;
            }

            self.repositories.remove(&v.id);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn repositories_set_manual_interaction(
        &mut self,
        owner: &str,
        name: &str,
        value: bool,
    ) -> Result<Repository> {
        let mut repository = self.repositories_get_expect(owner, name).await?;
        repository.manual_interaction = value;
        self.repositories.insert(repository.id, repository.clone());
        Ok(repository)
    }

    async fn repositories_set_pr_title_validation_regex(
        &mut self,
        owner: &str,
        name: &str,
        value: &str,
    ) -> Result<Repository> {
        let mut repository = self.repositories_get_expect(owner, name).await?;
        repository.pr_title_validation_regex = value.to_owned();
        self.repositories.insert(repository.id, repository.clone());
        Ok(repository)
    }

    async fn repositories_set_default_strategy(
        &mut self,
        owner: &str,
        name: &str,
        strategy: GhMergeStrategy,
    ) -> Result<Repository> {
        let mut repository = self.repositories_get_expect(owner, name).await?;
        repository.default_strategy = strategy;
        self.repositories.insert(repository.id, repository.clone());
        Ok(repository)
    }

    async fn repositories_set_default_needed_reviewers_count(
        &mut self,
        owner: &str,
        name: &str,
        count: u64,
    ) -> Result<Repository> {
        let mut repository = self.repositories_get_expect(owner, name).await?;
        repository.default_needed_reviewers_count = count;
        self.repositories.insert(repository.id, repository.clone());
        Ok(repository)
    }

    async fn repositories_set_default_automerge(
        &mut self,
        owner: &str,
        name: &str,
        value: bool,
    ) -> Result<Repository> {
        let mut repository = self.repositories_get_expect(owner, name).await?;
        repository.default_automerge = value;
        self.repositories.insert(repository.id, repository.clone());
        Ok(repository)
    }

    async fn repositories_set_default_enable_qa(
        &mut self,
        owner: &str,
        name: &str,
        value: bool,
    ) -> Result<Repository> {
        let mut repository = self.repositories_get_expect(owner, name).await?;
        repository.default_enable_qa = value;
        self.repositories.insert(repository.id, repository.clone());
        Ok(repository)
    }

    async fn repositories_set_default_enable_checks(
        &mut self,
        owner: &str,
        name: &str,
        value: bool,
    ) -> Result<Repository> {
        let mut repository = self.repositories_get_expect(owner, name).await?;
        repository.default_enable_checks = value;
        self.repositories.insert(repository.id, repository.clone());
        Ok(repository)
    }

    /////////////////////
    // Required reviewers

    async fn required_reviewers_create(
        &mut self,
        instance: RequiredReviewer,
    ) -> Result<RequiredReviewer> {
        self.pull_requests_get_from_id_expect(instance.pull_request_id)
            .await?;

        self.required_reviewers.insert(
            (instance.username.clone(), instance.pull_request_id),
            instance.clone(),
        );
        Ok(instance)
    }

    async fn required_reviewers_list(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
    ) -> Result<Vec<RequiredReviewer>> {
        if let Some(pr) = self.pull_requests_get(owner, name, number).await? {
            let mut values: Vec<_> = self
                .required_reviewers
                .values()
                .filter(|r| r.pull_request_id == pr.id)
                .cloned()
                .collect();
            values.sort_by(|a, b| {
                (a.pull_request_id, &a.username).cmp(&(b.pull_request_id, &b.username))
            });
            Ok(values)
        } else {
            Ok(vec![])
        }
    }

    async fn required_reviewers_get(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        username: &str,
    ) -> Result<Option<RequiredReviewer>> {
        if let Some(pr) = self.pull_requests_get(owner, name, number).await? {
            Ok(self
                .required_reviewers
                .values()
                .find(|r| r.username == username && r.pull_request_id == pr.id)
                .cloned())
        } else {
            Ok(None)
        }
    }

    async fn required_reviewers_delete(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        username: &str,
    ) -> Result<bool> {
        if let Some(r) = self
            .required_reviewers_get(owner, name, number, username)
            .await?
        {
            self.required_reviewers
                .remove(&(r.username, r.pull_request_id));
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn required_reviewers_all(&mut self) -> Result<Vec<RequiredReviewer>> {
        let mut values: Vec<_> = self.required_reviewers.values().cloned().collect();
        values.sort_by(|a, b| {
            (a.pull_request_id, &a.username).cmp(&(b.pull_request_id, &b.username))
        });
        Ok(values)
    }
}
