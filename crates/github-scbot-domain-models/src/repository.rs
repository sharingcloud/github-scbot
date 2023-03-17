use github_scbot_core::{
    config::Config,
    types::{pulls::GhMergeStrategy, repository::RepositoryPath},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Repository {
    pub id: u64,
    pub owner: String,
    pub name: String,
    pub manual_interaction: bool,
    pub pr_title_validation_regex: String,
    pub default_strategy: GhMergeStrategy,
    pub default_needed_reviewers_count: u64,
    pub default_automerge: bool,
    pub default_enable_qa: bool,
    pub default_enable_checks: bool,
}

impl Default for Repository {
    fn default() -> Self {
        Self {
            id: 0,
            owner: String::new(),
            name: String::new(),
            manual_interaction: false,
            pr_title_validation_regex: String::new(),
            default_strategy: GhMergeStrategy::Merge,
            default_needed_reviewers_count: 0,
            default_automerge: false,
            default_enable_qa: false,
            default_enable_checks: true,
        }
    }
}

impl Repository {
    pub fn path(&self) -> RepositoryPath {
        RepositoryPath::new_from_components(&self.owner, &self.name)
    }

    pub fn with_config(mut self, config: &Config) -> Self {
        self.default_strategy = (&config.default_merge_strategy)
            .try_into()
            .unwrap_or_default();
        self.default_needed_reviewers_count = config.default_needed_reviewers_count;
        self.pr_title_validation_regex = config.default_pr_title_validation_regex.clone();
        self
    }
}
