use github_scbot_core::types::{pulls::GhMergeStrategy, status::QaStatus};
use serde::{Deserialize, Serialize};

use crate::Repository;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PullRequest {
    pub id: u64,
    pub repository_id: u64,
    pub number: u64,
    pub qa_status: QaStatus,
    pub needed_reviewers_count: u64,
    pub status_comment_id: u64,
    pub checks_enabled: bool,
    pub automerge: bool,
    pub locked: bool,
    pub strategy_override: Option<GhMergeStrategy>,
}

impl PullRequest {
    pub fn with_repository(mut self, repository: &Repository) -> Self {
        self.repository_id = repository.id;
        self.automerge = repository.default_automerge;
        self.checks_enabled = repository.default_enable_checks;
        self.needed_reviewers_count = repository.default_needed_reviewers_count;
        self.qa_status = if repository.default_enable_qa {
            Default::default()
        } else {
            QaStatus::Skipped
        };
        self
    }
}
