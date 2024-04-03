pub(crate) mod add_pull_request_rule;
pub(crate) mod apply_pull_request_rules;
pub(crate) mod automerge_pull_request;
pub(crate) mod determine_pull_request_merge_strategy;
pub(crate) mod get_or_create_repository;
pub(crate) mod merge_pull_request;
pub(crate) mod process_pull_request_event;
pub(crate) mod process_pull_request_opened;
pub(crate) mod remove_pull_request_rule;
pub(crate) mod resolve_pull_request_rules;
pub(crate) mod set_step_label;
pub(crate) mod synchronize_pull_request;
pub(crate) mod synchronize_pull_request_and_update_status;
pub(crate) mod try_merge_pull_request_from_status;
pub(crate) mod update_step_label_from_status;

pub use add_pull_request_rule::AddPullRequestRuleInterface;
pub use apply_pull_request_rules::ApplyPullRequestRulesInterface;
pub use automerge_pull_request::AutomergePullRequestInterface;
pub use determine_pull_request_merge_strategy::DeterminePullRequestMergeStrategyInterface;
pub use get_or_create_repository::GetOrCreateRepositoryInterface;
pub use merge_pull_request::MergePullRequestInterface;
pub use process_pull_request_event::ProcessPullRequestEventInterface;
pub use process_pull_request_opened::ProcessPullRequestOpenedInterface;
pub use remove_pull_request_rule::RemovePullRequestRuleInterface;
pub use resolve_pull_request_rules::ResolvePullRequestRulesInterface;
pub use set_step_label::SetStepLabelInterface;
pub use synchronize_pull_request::SynchronizePullRequestInterface;
pub use synchronize_pull_request_and_update_status::SynchronizePullRequestAndUpdateStatusInterface;
pub use try_merge_pull_request_from_status::TryMergePullRequestFromStatusInterface;
pub use update_step_label_from_status::UpdateStepLabelFromStatusInterface;

#[cfg(any(test, feature = "testkit"))]
pub use self::{
    add_pull_request_rule::MockAddPullRequestRuleInterface,
    apply_pull_request_rules::MockApplyPullRequestRulesInterface,
    automerge_pull_request::MockAutomergePullRequestInterface,
    determine_pull_request_merge_strategy::MockDeterminePullRequestMergeStrategyInterface,
    get_or_create_repository::MockGetOrCreateRepositoryInterface,
    merge_pull_request::MockMergePullRequestInterface,
    process_pull_request_event::MockProcessPullRequestEventInterface,
    remove_pull_request_rule::MockRemovePullRequestRuleInterface,
    resolve_pull_request_rules::MockResolvePullRequestRulesInterface,
    set_step_label::MockSetStepLabelInterface,
    synchronize_pull_request::MockSynchronizePullRequestInterface,
    synchronize_pull_request_and_update_status::MockSynchronizePullRequestAndUpdateStatusInterface,
    try_merge_pull_request_from_status::MockTryMergePullRequestFromStatusInterface,
    update_step_label_from_status::MockUpdateStepLabelFromStatusInterface,
};
