use std::time::Duration;

use async_trait::async_trait;
use prbot_lock_interface::{LockError, LockInstance, LockService, LockStatus};
use redis::{Client, Cmd, Value};

/// Redis lock service.
#[derive(Clone)]
pub struct RedisLockService(Client);

impl RedisLockService {
    /// Creates a new redis adapter.
    pub fn new(addr: &str) -> Self {
        Self(Client::open(addr).unwrap_or_else(|_| panic!("Unsupported redis URL: {addr}")))
    }

    async fn execute_command(&self, cmd: &Cmd) -> Result<Value, LockError> {
        let mut conn = self
            .0
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| LockError::ImplementationError { source: e.into() })?;

        cmd.query_async(&mut conn)
            .await
            .map_err(|e| LockError::ImplementationError { source: e.into() })
    }
}

#[async_trait]
impl LockService for RedisLockService {
    #[tracing::instrument(skip(self), ret)]
    async fn try_lock_resource<'a>(&'a self, name: &str) -> Result<LockStatus<'a>, LockError> {
        let response = self
            .execute_command(
                redis::cmd("SET")
                    .arg(name)
                    .arg(1)
                    .arg("NX")
                    .arg("PX")
                    .arg(30000),
            )
            .await?;

        match response {
            Value::Okay => Ok(LockStatus::SuccessfullyLocked(LockInstance::new(
                self, name,
            ))),
            Value::Nil => Ok(LockStatus::AlreadyLocked),
            other => Err(LockError::ImplementationError {
                source: format!("Unsupported response: {other:?}").into(),
            }),
        }
    }

    #[tracing::instrument(skip(self), ret)]
    async fn has_resource(&self, name: &str) -> Result<bool, LockError> {
        let response = self.execute_command(redis::cmd("GET").arg(name)).await?;
        Ok(response != Value::Nil)
    }

    #[tracing::instrument(skip(self))]
    async fn del_resource(&self, name: &str) -> Result<(), LockError> {
        self.execute_command(redis::cmd("DEL").arg(name)).await?;
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
        timeout: Duration,
    ) -> Result<(), LockError> {
        self.execute_command(
            redis::cmd("SET")
                .arg(name)
                .arg(value)
                .arg("EX")
                .arg(timeout.as_secs()),
        )
        .await?;
        Ok(())
    }

    #[tracing::instrument(skip(self), ret)]
    async fn get_resource(&self, name: &str) -> Result<Option<String>, LockError> {
        let result = self.execute_command(redis::cmd("GET").arg(name)).await?;
        match result {
            Value::Nil => Ok(None),
            Value::Data(d) => Ok(Some(String::from_utf8_lossy(&d).into_owned())),
            _ => unreachable!(),
        }
    }

    #[tracing::instrument(skip(self))]
    async fn health_check(&self) -> Result<(), LockError> {
        self.execute_command(&redis::cmd("PING")).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::*;

    #[tokio::test]
    async fn test_redis() -> Result<(), Box<dyn Error>> {
        let lock_mgr = RedisLockService::new("redis://localhost");
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
