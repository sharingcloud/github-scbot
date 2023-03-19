use serde::{Deserialize, Serialize};

use crate::{MergeStrategy, RuleBranch};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct MergeRule {
    pub repository_id: u64,
    pub base_branch: RuleBranch,
    pub head_branch: RuleBranch,
    pub strategy: MergeStrategy,
}
