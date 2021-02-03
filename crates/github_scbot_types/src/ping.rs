//! Ping types.

use serde::Deserialize;

use super::common::{GHRepository, GHUser};

/// GitHub Ping event.
#[derive(Debug, Deserialize)]
pub struct GHPingEvent {
    /// Zen text.
    pub zen: String,
    /// Hook ID.
    pub hook_id: u64,
    /// Repository.
    pub repository: Option<GHRepository>,
    /// Sender.
    pub sender: Option<GHUser>,
}
