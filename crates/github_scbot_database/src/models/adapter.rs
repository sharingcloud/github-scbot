use super::{
    auth::{
        AccountDbAdapter, DummyAccountDbAdapter, DummyExternalAccountDbAdapter,
        DummyExternalAccountRightDbAdapter, ExternalAccountDbAdapter,
        ExternalAccountRightDbAdapter, IAccountDbAdapter, IExternalAccountDbAdapter,
        IExternalAccountRightDbAdapter,
    },
    history::{DummyHistoryWebhookDbAdapter, HistoryWebhookDbAdapter, IHistoryWebhookDbAdapter},
    merge_rule::{DummyMergeRuleDbAdapter, IMergeRuleDbAdapter, MergeRuleDbAdapter},
    pulls::{DummyPullRequestDbAdapter, IPullRequestDbAdapter, PullRequestDbAdapter},
    repository::{DummyRepositoryDbAdapter, IRepositoryDbAdapter, RepositoryDbAdapter},
    review::{DummyReviewDbAdapter, IReviewDbAdapter, ReviewDbAdapter},
};
use crate::DbPool;

/// Database adapter.
pub trait IDatabaseAdapter {
    /// Gets account DB adapter.
    fn account(&self) -> &dyn IAccountDbAdapter;
    /// Gets external account DB adapter.
    fn external_account(&self) -> &dyn IExternalAccountDbAdapter;
    /// Gets external account right DB adapter.
    fn external_account_right(&self) -> &dyn IExternalAccountRightDbAdapter;
    /// Gets history webhook DB adapter.
    fn history_webhook(&self) -> &dyn IHistoryWebhookDbAdapter;
    /// Gets merge rule DB adapter.
    fn merge_rule(&self) -> &dyn IMergeRuleDbAdapter;
    /// Gets pull request DB adapter.
    fn pull_request(&self) -> &dyn IPullRequestDbAdapter;
    /// Gets repository DB adapter.
    fn repository(&self) -> &dyn IRepositoryDbAdapter;
    /// Gets review DB adapter.
    fn review(&self) -> &dyn IReviewDbAdapter;
}

/// Concrete database adapter.
pub struct DatabaseAdapter<'a> {
    account_adapter: AccountDbAdapter<'a>,
    external_account_right_adapter: ExternalAccountRightDbAdapter<'a>,
    external_account_adapter: ExternalAccountDbAdapter<'a>,
    history_webhook_adapter: HistoryWebhookDbAdapter<'a>,
    merge_rule_adapter: MergeRuleDbAdapter<'a>,
    pull_request_adapter: PullRequestDbAdapter<'a>,
    repository_adapter: RepositoryDbAdapter<'a>,
    review_adapter: ReviewDbAdapter<'a>,
}

impl<'a> DatabaseAdapter<'a> {
    /// Creates a new database adapter.
    pub fn new(pool: &'a DbPool) -> Self {
        Self {
            account_adapter: AccountDbAdapter::new(pool),
            external_account_adapter: ExternalAccountDbAdapter::new(pool),
            external_account_right_adapter: ExternalAccountRightDbAdapter::new(pool),
            history_webhook_adapter: HistoryWebhookDbAdapter::new(pool),
            merge_rule_adapter: MergeRuleDbAdapter::new(pool),
            pull_request_adapter: PullRequestDbAdapter::new(pool),
            repository_adapter: RepositoryDbAdapter::new(pool),
            review_adapter: ReviewDbAdapter::new(pool),
        }
    }
}

impl<'a> IDatabaseAdapter for DatabaseAdapter<'a> {
    fn account(&self) -> &dyn IAccountDbAdapter {
        &self.account_adapter
    }

    fn external_account_right(&self) -> &dyn IExternalAccountRightDbAdapter {
        &self.external_account_right_adapter
    }

    fn external_account(&self) -> &dyn IExternalAccountDbAdapter {
        &self.external_account_adapter
    }

    fn history_webhook(&self) -> &dyn IHistoryWebhookDbAdapter {
        &self.history_webhook_adapter
    }

    fn merge_rule(&self) -> &dyn IMergeRuleDbAdapter {
        &self.merge_rule_adapter
    }

    fn pull_request(&self) -> &dyn IPullRequestDbAdapter {
        &self.pull_request_adapter
    }

    fn repository(&self) -> &dyn IRepositoryDbAdapter {
        &self.repository_adapter
    }

    fn review(&self) -> &dyn IReviewDbAdapter {
        &self.review_adapter
    }
}

/// Dummy database adapter.
#[derive(Default)]
pub struct DummyDatabaseAdapter {
    /// Account adapter.
    pub account_adapter: DummyAccountDbAdapter,
    /// External account right adapter.
    pub external_account_right_adapter: DummyExternalAccountRightDbAdapter,
    /// External account adapter.
    pub external_account_adapter: DummyExternalAccountDbAdapter,
    /// History webhook adapter.
    pub history_webhook_adapter: DummyHistoryWebhookDbAdapter,
    /// Merge rule adapter.
    pub merge_rule_adapter: DummyMergeRuleDbAdapter,
    /// Pull request adapter.
    pub pull_request_adapter: DummyPullRequestDbAdapter,
    /// Repository adapter.
    pub repository_adapter: DummyRepositoryDbAdapter,
    /// Review adapter.
    pub review_adapter: DummyReviewDbAdapter,
}

impl DummyDatabaseAdapter {
    /// Creates a new dummy database adapter.
    pub fn new() -> Self {
        Self::default()
    }
}

impl IDatabaseAdapter for DummyDatabaseAdapter {
    fn account(&self) -> &dyn IAccountDbAdapter {
        &self.account_adapter
    }

    fn external_account(&self) -> &dyn IExternalAccountDbAdapter {
        &self.external_account_adapter
    }

    fn external_account_right(&self) -> &dyn IExternalAccountRightDbAdapter {
        &self.external_account_right_adapter
    }

    fn history_webhook(&self) -> &dyn IHistoryWebhookDbAdapter {
        &self.history_webhook_adapter
    }

    fn merge_rule(&self) -> &dyn IMergeRuleDbAdapter {
        &self.merge_rule_adapter
    }

    fn pull_request(&self) -> &dyn IPullRequestDbAdapter {
        &self.pull_request_adapter
    }

    fn repository(&self) -> &dyn IRepositoryDbAdapter {
        &self.repository_adapter
    }

    fn review(&self) -> &dyn IReviewDbAdapter {
        &self.review_adapter
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{tests::using_test_db, DatabaseError};

    #[actix_rt::test]
    async fn test_dummy_adapter() {
        using_adapter(&DummyDatabaseAdapter::new()).await;
    }

    async fn using_adapter(adapter: &impl IDatabaseAdapter) {
        assert_eq!(
            adapter
                .external_account_right()
                .list_rights("test")
                .await
                .unwrap(),
            vec![]
        )
    }

    #[actix_rt::test]
    async fn test_db_adapter() {
        using_test_db("test_adapters", |_config, pool| async move {
            using_adapter(&DatabaseAdapter::new(&pool)).await;
            Ok::<_, DatabaseError>(())
        })
        .await
        .unwrap();
    }
}
