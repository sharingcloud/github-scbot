//! Redis interfaces.

use std::time::Duration;

use actix::MailboxError;
use async_trait::async_trait;
use thiserror::Error;

/// Lock error.
#[derive(Debug, Error, Clone)]
pub enum RedisError {
    /// Mailbox error.
    #[error("Actix mailbox error: {0}")]
    MailboxError(String),

    /// Actix error.
    #[error("Actix-Redis error: {0}")]
    ActixError(String),

    /// Command error.
    #[error("Redis command error: {0}")]
    CommandError(String),
}

impl From<MailboxError> for RedisError {
    fn from(err: MailboxError) -> Self {
        Self::MailboxError(err.to_string())
    }
}

impl From<actix_redis::Error> for RedisError {
    fn from(err: actix_redis::Error) -> Self {
        Self::ActixError(err.to_string())
    }
}

/// Lock status.
#[derive(Clone)]
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
}
