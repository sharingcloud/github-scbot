//! Utils.

#![warn(missing_docs)]
#![warn(clippy::all)]

mod tests;
use std::time::{SystemTime, UNIX_EPOCH};

pub use tests::Mock;

/// Time utilities.
pub struct TimeUtils;

impl TimeUtils {
    /// Get current timestamp.
    pub fn now_timestamp() -> u64 {
        let start = SystemTime::now();
        let duration = start.duration_since(UNIX_EPOCH).expect("time collapsed");

        duration.as_secs()
    }
}
