//! Redis interfaces.

use std::time::Duration;

use async_trait::async_trait;

use crate::{LockError, LockStatus};

/// Cache adapter trait.
#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait]
pub trait LockService: Send + Sync {
    /// Tries to lock a resource.
    async fn try_lock_resource<'a>(&'a self, name: &str) -> Result<LockStatus<'a>, LockError>;
    /// Checks if resource exists.
    async fn has_resource(&self, name: &str) -> Result<bool, LockError>;
    /// Deletes a resource if it exists.
    async fn del_resource(&self, name: &str) -> Result<(), LockError>;
    /// Sleep for duration.
    async fn sleep_for_duration(&self, duration: Duration) -> Result<(), LockError>;
    /// Sets a resource.
    async fn set_resource(
        &self,
        name: &str,
        value: &str,
        timeout: Duration,
    ) -> Result<(), LockError>;
    /// Gets a resource.
    async fn get_resource(&self, name: &str) -> Result<Option<String>, LockError>;

    /// Wait for a resource lock, until timeout.
    #[tracing::instrument(skip(self), ret)]
    async fn wait_lock_resource<'a>(
        &'a self,
        name: &str,
        timeout_ms: u64,
    ) -> Result<LockStatus<'a>, LockError> {
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
                self.sleep_for_duration(duration).await?;
                elapsed_time += millis;
            }
        }
    }
    /// Health check
    async fn health_check(&self) -> Result<(), LockError>;
}
