//! Redis wrappers.

use std::time::Duration;

use async_trait::async_trait;
use github_scbot_lock_interface::{LockError, LockService, LockStatus};
use github_scbot_lock_redis::RedisLockService;

use crate::metrics::REDIS_CALLS;

/// Redis service with metrics.
pub struct MetricsRedisService {
    inner: RedisLockService,
}

impl MetricsRedisService {
    /// Creates a new service.
    pub fn new(addr: &str) -> Self {
        Self {
            inner: RedisLockService::new(addr),
        }
    }
}

#[async_trait]
impl LockService for MetricsRedisService {
    async fn try_lock_resource<'a>(&'a self, name: &str) -> Result<LockStatus<'a>, LockError> {
        REDIS_CALLS.inc();
        self.inner.try_lock_resource(name).await
    }

    async fn has_resource(&self, name: &str) -> Result<bool, LockError> {
        REDIS_CALLS.inc();
        self.inner.has_resource(name).await
    }

    async fn del_resource(&self, name: &str) -> Result<(), LockError> {
        REDIS_CALLS.inc();
        self.inner.del_resource(name).await
    }

    async fn set_resource(
        &self,
        name: &str,
        value: &str,
        timeout: Duration,
    ) -> Result<(), LockError> {
        REDIS_CALLS.inc();
        self.inner.set_resource(name, value, timeout).await
    }

    async fn get_resource(&self, name: &str) -> Result<Option<String>, LockError> {
        REDIS_CALLS.inc();
        self.inner.get_resource(name).await
    }

    async fn health_check(&self) -> Result<(), LockError> {
        REDIS_CALLS.inc_by(2);
        self.inner.health_check().await
    }

    async fn sleep_for_duration(&self, duration: Duration) -> Result<(), LockError> {
        self.inner.sleep_for_duration(duration).await
    }
}
