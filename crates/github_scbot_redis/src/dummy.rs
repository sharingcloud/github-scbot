use async_trait::async_trait;
use github_scbot_utils::Mock;

use crate::interface::{IRedisAdapter, LockStatus, RedisError};

/// Dummy redis adapter.
pub struct DummyRedisAdapter<'a> {
    /// Try lock resource response.
    pub try_lock_resource_response: Mock<String, Result<LockStatus<'a>, RedisError>>,
    /// Has resource response.
    pub has_resource_response: Mock<String, Result<bool, RedisError>>,
    /// Del resource response.
    pub del_resource_response: Mock<String, Result<(), RedisError>>,
}

impl<'a> Default for DummyRedisAdapter<'a> {
    fn default() -> Self {
        Self {
            try_lock_resource_response: Mock::new(Box::new(|_name| Ok(LockStatus::AlreadyLocked))),
            has_resource_response: Mock::new(Box::new(|_name| Ok(false))),
            del_resource_response: Mock::new(Box::new(|_name| Ok(()))),
        }
    }
}

impl<'a> DummyRedisAdapter<'a> {
    /// Creates a dummy redis adapter.
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
#[allow(unused_variables)]
impl<'a> IRedisAdapter for DummyRedisAdapter<'a> {
    async fn try_lock_resource<'b>(&'b self, name: &str) -> Result<LockStatus<'b>, RedisError> {
        self.try_lock_resource_response.call(name.to_owned())
    }

    async fn has_resource(&self, name: &str) -> Result<bool, RedisError> {
        self.has_resource_response.call(name.to_owned())
    }

    async fn del_resource(&self, name: &str) -> Result<(), RedisError> {
        self.del_resource_response.call(name.to_owned())
    }
}
