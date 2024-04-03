pub(crate) mod add_merge_rule;
pub(crate) mod rename_repository;

pub use add_merge_rule::AddMergeRuleInterface;
pub use rename_repository::RenameRepositoryInterface;

#[cfg(any(test, feature = "testkit"))]
pub use self::{
    add_merge_rule::MockAddMergeRuleInterface, rename_repository::MockRenameRepositoryInterface,
};
