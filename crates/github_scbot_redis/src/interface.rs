//! Redis interfaces.

use github_scbot_libs::{actix::MailboxError, actix_redis, async_trait::async_trait};
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
    pub(crate) lock: &'a dyn IRedisAdapter,
    pub(crate) name: String,
}

impl<'a> LockInstance<'a> {
    /// Release lock instance.
    pub async fn release(self) -> Result<(), RedisError> {
        if self.lock.has_resource(&self.name).await? {
            self.lock.del_resource(&self.name).await?;
        }

        Ok(())
    }
}

/// Redis adapter trait.
#[async_trait]
pub trait IRedisAdapter: Sync {
    /// Tries to lock a resource.
    async fn try_lock_resource<'a>(&'a self, name: &str) -> Result<LockStatus<'a>, RedisError>;
    /// Checks if resource exists.
    async fn has_resource(&self, name: &str) -> Result<bool, RedisError>;
    /// Deletes a resource if it exists.
    async fn del_resource(&self, name: &str) -> Result<(), RedisError>;
}
