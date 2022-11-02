//! Redis interfaces.

use std::time::Duration;

use async_trait::async_trait;

use crate::{LockStatus, RedisError};

/// Redis adapter trait.
#[mockall::automock]
#[async_trait]
pub trait RedisService: Send + Sync {
    /// Tries to lock a resource.
    async fn try_lock_resource<'a>(&'a self, name: &str) -> Result<LockStatus<'a>, RedisError>;
    /// Checks if resource exists.
    async fn has_resource(&self, name: &str) -> Result<bool, RedisError>;
    /// Deletes a resource if it exists.
    async fn del_resource(&self, name: &str) -> Result<(), RedisError>;

    /// Wait for a resource lock, until timeout.
    #[tracing::instrument(skip(self), ret)]
    async fn wait_lock_resource<'a>(
        &'a self,
        name: &str,
        timeout_ms: u64,
    ) -> Result<LockStatus<'a>, RedisError> {
        // Try each 100ms
        let mut elapsed_time = 0;
        let millis = 100;
        let duration = Duration::from_millis(millis);

        loop {
            match self.try_lock_resource(name).await? {
                l @ LockStatus::SuccessfullyLocked(_) => return Ok(l),
                LockStatus::AlreadyLocked => (),
            }

            if elapsed_time > timeout_ms {
                return Ok(LockStatus::AlreadyLocked);
            } else {
                actix::clock::sleep(duration).await;
                elapsed_time += millis;
            }
        }
    }
    /// Health check
    async fn health_check(&self) -> Result<(), RedisError>;
}
