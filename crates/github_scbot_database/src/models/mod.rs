//! Database models.

mod adapter;
mod auth;
mod history;
mod merge_rule;
mod pulls;
mod repository;
mod review;

pub use adapter::{DatabaseAdapter, DummyDatabaseAdapter, IDatabaseAdapter};
pub use auth::{
    AccountDbAdapter, AccountModel, DummyAccountDbAdapter, DummyExternalAccountDbAdapter,
    DummyExternalAccountRightDbAdapter, ExternalAccountDbAdapter, ExternalAccountModel,
    ExternalAccountRightDbAdapter, ExternalAccountRightModel, ExternalJwtClaims, IAccountDbAdapter,
    IExternalAccountDbAdapter, IExternalAccountRightDbAdapter,
};
pub use history::{
    DummyHistoryWebhookDbAdapter, HistoryWebhookDbAdapter, HistoryWebhookModel,
    IHistoryWebhookDbAdapter,
};
pub use merge_rule::{
    DummyMergeRuleDbAdapter, IMergeRuleDbAdapter, MergeRuleDbAdapter, MergeRuleModel, RuleBranch,
};
pub use pulls::{
    DummyPullRequestDbAdapter, IPullRequestDbAdapter, PullRequestDbAdapter, PullRequestModel,
};
pub use repository::{
    DummyRepositoryDbAdapter, IRepositoryDbAdapter, RepositoryDbAdapter, RepositoryModel,
};
pub use review::{DummyReviewDbAdapter, IReviewDbAdapter, ReviewDbAdapter, ReviewModel};
