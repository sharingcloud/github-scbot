//! Redis wrappers.

use async_trait::async_trait;
use github_scbot_redis::{LockStatus, RedisError, RedisService, RedisServiceImpl};

use crate::metrics::REDIS_CALLS;

/// Redis service with metrics.
pub struct MetricsRedisService {
    inner: RedisServiceImpl,
}

impl MetricsRedisService {
    /// Creates a new service.
    pub fn new<T: Into<String>>(addr: T) -> Self {
        Self {
            inner: RedisServiceImpl::new(addr),
        }
    }
}

#[async_trait]
impl RedisService for MetricsRedisService {
    async fn try_lock_resource<'a>(&'a self, name: &str) -> Result<LockStatus<'a>, RedisError> {
        REDIS_CALLS.inc();
        self.inner.try_lock_resource(name).await
    }

    async fn has_resource(&self, name: &str) -> Result<bool, RedisError> {
        REDIS_CALLS.inc();
        self.inner.has_resource(name).await
    }

    async fn del_resource(&self, name: &str) -> Result<(), RedisError> {
        REDIS_CALLS.inc();
        self.inner.del_resource(name).await
    }

    async fn health_check(&self) -> Result<(), RedisError> {
        REDIS_CALLS.inc_by(2);
        self.inner.health_check().await
    }
}
