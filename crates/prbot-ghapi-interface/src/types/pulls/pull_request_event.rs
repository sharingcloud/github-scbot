use serde::{Deserialize, Serialize};

use super::{GhPullRequest, GhPullRequestAction};
use crate::types::common::{GhLabel, GhRepository, GhUser};

/// GitHub Pull request event.
#[derive(Debug, Deserialize, Serialize, Default, Clone, Eq, PartialEq)]
pub struct GhPullRequestEvent {
    /// Action.
    pub action: GhPullRequestAction,
    /// Number.
    pub number: u64,
    /// Pull request.
    pub pull_request: GhPullRequest,
    /// Label.
    pub label: Option<GhLabel>,
    /// Requested reviewer.
    pub requested_reviewer: Option<GhUser>,
    /// Repository.
    pub repository: GhRepository,
    /// Organization.
    pub organization: Option<GhUser>,
    /// Sender.
    pub sender: GhUser,
}
