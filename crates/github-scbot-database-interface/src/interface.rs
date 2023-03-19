use async_trait::async_trait;
use github_scbot_domain_models::{
    Account, ExternalAccount, ExternalAccountRight, MergeRule, MergeStrategy, PullRequest,
    QaStatus, Repository, RequiredReviewer, RuleBranch,
};

use crate::{DatabaseError, Result};

#[async_trait]
pub trait DbService: Send + Sync {
    async fn accounts_create(&mut self, instance: Account) -> Result<Account>;
    async fn accounts_update(&mut self, instance: Account) -> Result<Account>;
    async fn accounts_all(&mut self) -> Result<Vec<Account>>;
    async fn accounts_get(&mut self, username: &str) -> Result<Option<Account>>;
    async fn accounts_get_expect(&mut self, username: &str) -> Result<Account> {
        self.accounts_get(username)
            .await?
            .ok_or_else(|| DatabaseError::UnknownAccount(username.into()))
    }
    async fn accounts_delete(&mut self, username: &str) -> Result<bool>;
    async fn accounts_list_admins(&mut self) -> Result<Vec<Account>>;
    async fn accounts_set_is_admin(&mut self, username: &str, value: bool) -> Result<Account>;
    async fn external_account_rights_create(
        &mut self,
        instance: ExternalAccountRight,
    ) -> Result<ExternalAccountRight>;
    async fn external_account_rights_get(
        &mut self,
        owner: &str,
        name: &str,
        username: &str,
    ) -> Result<Option<ExternalAccountRight>>;
    async fn external_account_rights_delete(
        &mut self,
        owner: &str,
        name: &str,
        username: &str,
    ) -> Result<bool>;
    async fn external_account_rights_delete_all(&mut self, username: &str) -> Result<bool>;
    async fn external_account_rights_list(
        &mut self,
        username: &str,
    ) -> Result<Vec<ExternalAccountRight>>;
    async fn external_account_rights_all(&mut self) -> Result<Vec<ExternalAccountRight>>;
    async fn external_accounts_create(
        &mut self,
        instance: ExternalAccount,
    ) -> Result<ExternalAccount>;
    async fn external_accounts_update(
        &mut self,
        instance: ExternalAccount,
    ) -> Result<ExternalAccount>;
    async fn external_accounts_get(&mut self, username: &str) -> Result<Option<ExternalAccount>>;
    async fn external_accounts_get_expect(&mut self, username: &str) -> Result<ExternalAccount> {
        self.external_accounts_get(username)
            .await?
            .ok_or_else(|| DatabaseError::UnknownExternalAccount(username.into()))
    }
    async fn external_accounts_delete(&mut self, username: &str) -> Result<bool>;
    async fn external_accounts_all(&mut self) -> Result<Vec<ExternalAccount>>;
    async fn external_accounts_set_keys(
        &mut self,
        username: &str,
        public_key: &str,
        private_key: &str,
    ) -> Result<ExternalAccount>;
    async fn health_check(&mut self) -> Result<()>;
    async fn merge_rules_create(&mut self, instance: MergeRule) -> Result<MergeRule>;
    async fn merge_rules_update(&mut self, instance: MergeRule) -> Result<MergeRule>;
    async fn merge_rules_get(
        &mut self,
        owner: &str,
        name: &str,
        base_branch: RuleBranch,
        head_branch: RuleBranch,
    ) -> Result<Option<MergeRule>>;
    async fn merge_rules_get_expect(
        &mut self,
        owner: &str,
        name: &str,
        base_branch: RuleBranch,
        head_branch: RuleBranch,
    ) -> Result<MergeRule> {
        self.merge_rules_get(owner, name, base_branch.clone(), head_branch.clone())
            .await?
            .ok_or(DatabaseError::UnknownMergeRule(base_branch, head_branch))
    }
    async fn merge_rules_delete(
        &mut self,
        owner: &str,
        name: &str,
        base_branch: RuleBranch,
        head_branch: RuleBranch,
    ) -> Result<bool>;
    async fn merge_rules_all(&mut self) -> Result<Vec<MergeRule>>;
    async fn merge_rules_list(&mut self, owner: &str, name: &str) -> Result<Vec<MergeRule>>;
    async fn pull_requests_create(&mut self, instance: PullRequest) -> Result<PullRequest>;
    async fn pull_requests_update(&mut self, instance: PullRequest) -> Result<PullRequest>;
    async fn pull_requests_get(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
    ) -> Result<Option<PullRequest>>;
    async fn pull_requests_get_expect(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
    ) -> Result<PullRequest> {
        self.pull_requests_get(owner, name, number)
            .await?
            .ok_or_else(|| DatabaseError::UnknownPullRequest(format!("{owner}/{name}"), number))
    }
    async fn pull_requests_get_from_id(&mut self, id: u64) -> Result<Option<PullRequest>>;
    async fn pull_requests_get_from_id_expect(&mut self, id: u64) -> Result<PullRequest> {
        self.pull_requests_get_from_id(id)
            .await?
            .ok_or(DatabaseError::UnknownPullRequestId(id))
    }
    async fn pull_requests_delete(&mut self, owner: &str, name: &str, number: u64) -> Result<bool>;
    async fn pull_requests_list(&mut self, owner: &str, name: &str) -> Result<Vec<PullRequest>>;
    async fn pull_requests_all(&mut self) -> Result<Vec<PullRequest>>;
    async fn pull_requests_set_qa_status(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        status: QaStatus,
    ) -> Result<PullRequest>;
    async fn pull_requests_set_needed_reviewers_count(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        count: u64,
    ) -> Result<PullRequest>;
    async fn pull_requests_set_status_comment_id(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        id: u64,
    ) -> Result<PullRequest>;
    async fn pull_requests_set_checks_enabled(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        value: bool,
    ) -> Result<PullRequest>;
    async fn pull_requests_set_automerge(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        value: bool,
    ) -> Result<PullRequest>;
    async fn pull_requests_set_locked(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        value: bool,
    ) -> Result<PullRequest>;
    async fn pull_requests_set_strategy_override(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        strategy: Option<MergeStrategy>,
    ) -> Result<PullRequest>;
    async fn repositories_create(&mut self, instance: Repository) -> Result<Repository>;
    async fn repositories_update(&mut self, instance: Repository) -> Result<Repository>;
    async fn repositories_all(&mut self) -> Result<Vec<Repository>>;
    async fn repositories_get(&mut self, owner: &str, name: &str) -> Result<Option<Repository>>;
    async fn repositories_get_expect(&mut self, owner: &str, name: &str) -> Result<Repository> {
        self.repositories_get(owner, name)
            .await?
            .ok_or_else(|| DatabaseError::UnknownRepository(format!("{owner}/{name}")))
    }
    async fn repositories_get_from_id(&mut self, id: u64) -> Result<Option<Repository>>;
    async fn repositories_get_from_id_expect(&mut self, id: u64) -> Result<Repository> {
        self.repositories_get_from_id(id)
            .await?
            .ok_or(DatabaseError::UnknownRepositoryId(id))
    }
    async fn repositories_delete(&mut self, owner: &str, name: &str) -> Result<bool>;
    async fn repositories_set_manual_interaction(
        &mut self,
        owner: &str,
        name: &str,
        value: bool,
    ) -> Result<Repository>;
    async fn repositories_set_pr_title_validation_regex(
        &mut self,
        owner: &str,
        name: &str,
        value: &str,
    ) -> Result<Repository>;
    async fn repositories_set_default_strategy(
        &mut self,
        owner: &str,
        name: &str,
        strategy: MergeStrategy,
    ) -> Result<Repository>;
    async fn repositories_set_default_needed_reviewers_count(
        &mut self,
        owner: &str,
        name: &str,
        count: u64,
    ) -> Result<Repository>;
    async fn repositories_set_default_automerge(
        &mut self,
        owner: &str,
        name: &str,
        value: bool,
    ) -> Result<Repository>;
    async fn repositories_set_default_enable_qa(
        &mut self,
        owner: &str,
        name: &str,
        value: bool,
    ) -> Result<Repository>;
    async fn repositories_set_default_enable_checks(
        &mut self,
        owner: &str,
        name: &str,
        value: bool,
    ) -> Result<Repository>;
    async fn required_reviewers_create(
        &mut self,
        instance: RequiredReviewer,
    ) -> Result<RequiredReviewer>;
    async fn required_reviewers_list(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
    ) -> Result<Vec<RequiredReviewer>>;
    async fn required_reviewers_get(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        username: &str,
    ) -> Result<Option<RequiredReviewer>>;
    async fn required_reviewers_delete(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        username: &str,
    ) -> Result<bool>;
    async fn required_reviewers_all(&mut self) -> Result<Vec<RequiredReviewer>>;
}
