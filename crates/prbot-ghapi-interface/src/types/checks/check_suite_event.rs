use serde::{Deserialize, Serialize};

use super::{GhCheckSuite, GhCheckSuiteAction};
use crate::types::common::{GhRepository, GhUser};

/// GitHub Check suite event.
#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct GhCheckSuiteEvent {
    /// Action.
    pub action: GhCheckSuiteAction,
    /// Check suite.
    pub check_suite: GhCheckSuite,
    /// Repository.
    pub repository: GhRepository,
    /// Organization.
    pub organization: Option<GhUser>,
    /// Sender.
    pub sender: GhUser,
}
