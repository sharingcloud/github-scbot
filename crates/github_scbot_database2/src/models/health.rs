use crate::{DatabaseError, Result};
use async_trait::async_trait;
use sqlx::{PgConnection, PgPool};

#[async_trait]
#[mockall::automock]
pub trait HealthDB {
    async fn health_check(&mut self) -> Result<()>;
}

pub struct HealthDBImpl<'a> {
    connection: &'a mut PgConnection,
}

impl<'a> HealthDBImpl<'a> {
    pub fn new(connection: &'a mut PgConnection) -> Self {
        Self { connection }
    }
}

pub struct HealthDBImplPool {
    pool: PgPool,
}

impl HealthDBImplPool {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl HealthDB for HealthDBImplPool {
    async fn health_check(&mut self) -> Result<()> {
        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(DatabaseError::ConnectionError)?;
        let data = HealthDBImpl::new(&mut *conn).health_check().await?;
        Ok(data)
    }
}

#[async_trait]
impl<'a> HealthDB for HealthDBImpl<'a> {
    async fn health_check(&mut self) -> Result<()> {
        sqlx::query("SELECT 1;")
            .execute(&mut *self.connection)
            .await
            .map_err(DatabaseError::SqlError)?;

        Ok(())
    }
}
