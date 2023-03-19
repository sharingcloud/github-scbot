use serde::Deserialize;

use crate::types::common::{GhRepository, GhUser};

/// GitHub Ping event.
#[derive(Debug, Deserialize, Default, PartialEq, Eq)]
pub struct GhPingEvent {
    /// Zen text.
    pub zen: String,
    /// Hook ID.
    pub hook_id: u64,
    /// Repository.
    pub repository: Option<GhRepository>,
    /// Sender.
    pub sender: Option<GhUser>,
}
