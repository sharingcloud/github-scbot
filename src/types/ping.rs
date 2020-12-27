//! Webhook ping types

use serde::Deserialize;

use super::common::{Repository, User};

#[derive(Debug, Deserialize)]
pub struct PingEvent {
    pub zen: String,
    pub hook_id: u64,
    pub repository: Option<Repository>,
    pub sender: Option<User>,
}
