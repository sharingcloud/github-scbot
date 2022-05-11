//! Redis interfaces.

use std::time::Duration;

use actix::MailboxError;
use async_trait::async_trait;
use snafu::{prelude::*, Backtrace};

/// Lock error.
#[allow(missing_docs)]
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum RedisError {
    /// Mailbox error.
    #[snafu(display("Actix mailbox error,\n  caused by: {}", source))]
    MailboxError {
        source: MailboxError,
        backtrace: Backtrace,
    },

    /// Actix error.
    #[snafu(display("Actix-Redis error,\n  caused by: {}", source))]
    ActixError {
        source: actix_redis::Error,
        backtrace: Backtrace,
    },

    /// Command error.
    #[snafu(display("Redis command error: {}", result))]
    CommandError {
        result: String,
        backtrace: Backtrace,
    },
}

/// Lock status.
#[derive(Clone, Debug)]
pub enum LockStatus<'a> {
    /// Already locked.
    AlreadyLocked,
    /// Lock successful.
    SuccessfullyLocked(LockInstance<'a>),
}

/// Lock instance.
#[derive(Clone)]
#[must_use]
pub struct LockInstance<'a> {
    pub(crate) lock: Option<&'a dyn RedisService>,
    pub(crate) name: String,
}

impl<'a> std::fmt::Debug for LockInstance<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LockInstance")
            .field("name", &self.name)
            .finish()
    }
}

impl<'a> LockInstance<'a> {
    /// Create a new dummy lock.
    pub fn new_dummy<T: Into<String>>(name: T) -> Self {
        Self {
            lock: None,
            name: name.into(),
        }
    }

    /// Release lock instance.
    pub async fn release(self) -> Result<(), RedisError> {
        if let Some(lock) = self.lock {
            if lock.has_resource(&self.name).await? {
                lock.del_resource(&self.name).await?;
            }
        }

        Ok(())
    }
}

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
