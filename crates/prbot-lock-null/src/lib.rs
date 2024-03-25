use std::time::Duration;

use async_trait::async_trait;
use prbot_lock_interface::{LockError, LockInstance, LockService, LockStatus};

/// Redis lock service.
#[derive(Clone, Default)]
pub struct NullLockService {
    _private: (),
}

impl NullLockService {
    /// Creates a null lock service.
    pub fn new() -> Self {
        Self { _private: () }
    }
}

#[async_trait]
impl LockService for NullLockService {
    #[tracing::instrument(skip(self), ret)]
    async fn try_lock_resource<'a>(&'a self, name: &str) -> Result<LockStatus<'a>, LockError> {
        Ok(LockStatus::SuccessfullyLocked(LockInstance::new(
            self, name,
        )))
    }

    #[tracing::instrument(skip(self), ret)]
    async fn has_resource(&self, name: &str) -> Result<bool, LockError> {
        Ok(false)
    }

    #[tracing::instrument(skip(self))]
    async fn del_resource(&self, name: &str) -> Result<(), LockError> {
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn sleep_for_duration(&self, duration: Duration) -> Result<(), LockError> {
        tokio::time::sleep(duration).await;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn set_resource(
        &self,
        name: &str,
        value: &str,
        _timeout: Duration,
    ) -> Result<(), LockError> {
        Ok(())
    }

    #[tracing::instrument(skip(self), ret)]
    async fn get_resource(&self, name: &str) -> Result<Option<String>, LockError> {
        Ok(None)
    }

    #[tracing::instrument(skip(self))]
    async fn health_check(&self) -> Result<(), LockError> {
        Ok(())
    }
}
