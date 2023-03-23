mod automerge_pull_request;
mod determine_pull_request_merge_strategy;
mod get_or_create_repository;
mod handle_pull_request_event;
mod merge_pull_request;
mod process_pull_request_opened;
mod set_step_label;
mod synchronize_pull_request;
mod synchronize_pull_request_and_update_status;

pub use automerge_pull_request::{
    AutomergePullRequestUseCase, AutomergePullRequestUseCaseInterface,
};
pub use determine_pull_request_merge_strategy::DeterminePullRequestMergeStrategyUseCase;
pub use get_or_create_repository::GetOrCreateRepositoryUseCase;
pub use handle_pull_request_event::HandlePullRequestEventUseCase;
pub use merge_pull_request::{MergePullRequestUseCase, MergePullRequestUseCaseInterface};
pub use process_pull_request_opened::{
    ProcessPullRequestOpenedUseCase, ProcessPullRequestOpenedUseCaseInterface,
};
pub use set_step_label::{SetStepLabelUseCase, SetStepLabelUseCaseInterface};
pub use synchronize_pull_request::{
    SynchronizePullRequestUseCase, SynchronizePullRequestUseCaseInterface,
};
pub use synchronize_pull_request_and_update_status::SynchronizePullRequestAndUpdateStatusUseCase;

#[cfg(any(test, feature = "testkit"))]
pub use self::{
    automerge_pull_request::MockAutomergePullRequestUseCaseInterface,
    merge_pull_request::MockMergePullRequestUseCaseInterface,
    set_step_label::MockSetStepLabelUseCaseInterface,
    synchronize_pull_request::MockSynchronizePullRequestUseCaseInterface,
};
