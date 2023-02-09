use github_scbot_core::types::{pulls::GhMergeStrategy, rule_branch::RuleBranch};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct MergeRule {
    pub repository_id: u64,
    pub base_branch: RuleBranch,
    pub head_branch: RuleBranch,
    pub strategy: GhMergeStrategy,
}
