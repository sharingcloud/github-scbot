mod gif;
mod help;
mod is_admin;
mod lock;
mod merge;
mod ping;
mod set_automerge;
mod set_checks_status;
mod set_labels;
mod set_merge_strategy;
mod set_qa_status;
mod set_reviewers;

pub use gif::GifCommand;
pub use help::HelpCommand;
pub use is_admin::IsAdminCommand;
pub use lock::LockCommand;
pub use merge::MergeCommand;
pub use ping::PingCommand;
pub use set_automerge::SetAutomergeCommand;
pub use set_checks_status::SetChecksStatusCommand;
pub use set_labels::SetLabelsCommand;
pub use set_merge_strategy::SetMergeStrategyCommand;
pub use set_qa_status::SetQaStatusCommand;
pub use set_reviewers::SetReviewersCommand;
