use actix::Addr;
use actix_redis::{Command, RedisActor, RespValue};
use async_trait::async_trait;
use redis_async::resp_array;

use crate::{LockError, LockInstance, LockService, LockStatus};

/// Redis adapter.
#[derive(Clone)]
pub struct RedisServiceImpl(Addr<RedisActor>);

impl RedisServiceImpl {
    /// Creates a new redis adapter.
    pub fn new<T: Into<String>>(addr: T) -> Self {
        Self(RedisActor::start(addr))
    }

    async fn execute_command(&self, value: RespValue) -> Result<RespValue, LockError> {
        let response = self
            .0
            .send(Command(value))
            .await
            .map_err(|e| LockError::MailboxError { source: e })?
            .map_err(|e| LockError::ActixError { source: e })?;

        match response {
            RespValue::Error(e) => Err(LockError::CommandError { result: e }),
            v => Ok(v),
        }
    }
}

#[async_trait]
impl LockService for RedisServiceImpl {
    #[tracing::instrument(skip(self), ret)]
    async fn try_lock_resource<'a>(&'a self, name: &str) -> Result<LockStatus<'a>, LockError> {
        let response = self
            .execute_command(resp_array!["SET", name, "1", "NX", "PX", "30000"])
            .await?;

        match response {
            RespValue::SimpleString(s) => {
                if &s == "OK" {
                    Ok(LockStatus::SuccessfullyLocked(LockInstance {
                        lock: Some(self),
                        name: name.to_owned(),
                    }))
                } else {
                    Err(LockError::CommandError {
                        result: format!("Unsupported response: {:?}", RespValue::SimpleString(s)),
                    })
                }
            }
            RespValue::Nil => Ok(LockStatus::AlreadyLocked),
            v => Err(LockError::CommandError {
                result: format!("Unsupported response: {:?}", v),
            }),
        }
    }

    async fn has_resource(&self, name: &str) -> Result<bool, LockError> {
        let response = self.execute_command(resp_array!["GET", name]).await?;
        Ok(response != RespValue::Nil)
    }

    async fn del_resource(&self, name: &str) -> Result<(), LockError> {
        self.execute_command(resp_array!["DEL", name]).await?;
        Ok(())
    }

    async fn health_check(&self) -> Result<(), LockError> {
        self.execute_command(resp_array!["PING"]).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::*;

    #[actix_rt::test]
    async fn test_redis() -> Result<(), Box<dyn Error>> {
        let lock_mgr = RedisServiceImpl::new("127.0.0.1:6379");
        let key = "this-is-a-test";

        lock_mgr.del_resource(key).await?;

        if let LockStatus::SuccessfullyLocked(s) = lock_mgr.try_lock_resource(key).await? {
            assert!(matches!(
                lock_mgr.try_lock_resource(key).await?,
                LockStatus::AlreadyLocked
            ));

            s.release().await?;
        }

        let status = lock_mgr.try_lock_resource(key).await?;
        assert!(matches!(status, LockStatus::SuccessfullyLocked(_)));

        Ok(())
    }
}
