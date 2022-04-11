use async_trait::async_trait;
use github_scbot_database_macros::SCGetter;
use sqlx::{postgres::PgRow, FromRow, PgConnection, PgPool, Postgres, Row, Transaction};

use crate::{errors::Result, DatabaseError};

#[derive(SCGetter, Debug, Clone, Default, derive_builder::Builder)]
#[builder(default)]
pub struct ExternalAccount {
    #[get_ref]
    username: String,
    #[get_ref]
    public_key: String,
    #[get_ref]
    private_key: String,
}

impl ExternalAccount {
    pub fn builder() -> ExternalAccountBuilder {
        ExternalAccountBuilder::default()
    }
}

impl<'r> FromRow<'r, PgRow> for ExternalAccount {
    fn from_row(row: &'r PgRow) -> core::result::Result<Self, sqlx::Error> {
        Ok(Self {
            username: row.try_get("username")?,
            public_key: row.try_get("public_key")?,
            private_key: row.try_get("private_key")?,
        })
    }
}

#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait ExternalAccountDB {
    async fn create(&mut self, instance: ExternalAccount) -> Result<ExternalAccount>;
    async fn get(&mut self, username: &str) -> Result<Option<ExternalAccount>>;
    async fn delete(&mut self, username: &str) -> Result<bool>;
    async fn set_keys(
        &mut self,
        username: &str,
        public_key: &str,
        private_key: &str,
    ) -> Result<ExternalAccount>;
}

pub struct ExternalAccountDBImpl<'a> {
    connection: &'a mut PgConnection,
}

impl<'a> ExternalAccountDBImpl<'a> {
    pub fn new(connection: &'a mut PgConnection) -> Self {
        Self { connection }
    }
}

pub struct ExternalAccountDBImplPool {
    pool: PgPool,
}

impl ExternalAccountDBImplPool {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn begin<'a>(&mut self) -> Result<Transaction<'a, Postgres>> {
        self.pool
            .begin()
            .await
            .map_err(DatabaseError::ConnectionError)
    }

    pub async fn commit<'a>(&mut self, transaction: Transaction<'a, Postgres>) -> Result<()> {
        transaction
            .commit()
            .await
            .map_err(DatabaseError::TransactionError)
    }
}

#[async_trait]
impl ExternalAccountDB for ExternalAccountDBImplPool {
    async fn create(&mut self, instance: ExternalAccount) -> Result<ExternalAccount> {
        let mut transaction = self.begin().await?;
        let data = ExternalAccountDBImpl::new(&mut *transaction)
            .create(instance)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn get(&mut self, username: &str) -> Result<Option<ExternalAccount>> {
        let mut transaction = self.begin().await?;
        let data = ExternalAccountDBImpl::new(&mut *transaction)
            .get(username)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn delete(&mut self, username: &str) -> Result<bool> {
        let mut transaction = self.begin().await?;
        let data = ExternalAccountDBImpl::new(&mut *transaction)
            .delete(username)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn set_keys(
        &mut self,
        username: &str,
        public_key: &str,
        private_key: &str,
    ) -> Result<ExternalAccount> {
        let mut transaction = self.begin().await?;
        let data = ExternalAccountDBImpl::new(&mut *transaction)
            .set_keys(username, public_key, private_key)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }
}

#[async_trait]
impl<'a> ExternalAccountDB for ExternalAccountDBImpl<'a> {
    async fn create(&mut self, instance: ExternalAccount) -> Result<ExternalAccount> {
        let username: String = sqlx::query(
            r#"
            INSERT INTO external_account
            (
                username,
                public_key,
                private_key
            ) VALUES (
                $1,
                $2,
                $3
            )
            RETURNING username;
            "#,
        )
        .bind(instance.username)
        .bind(instance.public_key)
        .bind(instance.private_key)
        .fetch_one(&mut *self.connection)
        .await
        .map_err(DatabaseError::SqlError)?
        .get(0);

        self.get(&username).await.map(|x| x.unwrap())
    }

    async fn get(&mut self, username: &str) -> Result<Option<ExternalAccount>> {
        sqlx::query_as::<_, ExternalAccount>(
            r#"
                SELECT *
                FROM external_account
                WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_optional(&mut *self.connection)
        .await
        .map_err(DatabaseError::SqlError)
    }

    async fn delete(&mut self, username: &str) -> Result<bool> {
        sqlx::query(
            r#"
            DELETE FROM external_account
            WHERE username = $1
        "#,
        )
        .bind(username)
        .execute(&mut *self.connection)
        .await
        .map(|x| x.rows_affected() > 0)
        .map_err(DatabaseError::SqlError)
    }

    async fn set_keys(
        &mut self,
        username: &str,
        public_key: &str,
        private_key: &str,
    ) -> Result<ExternalAccount> {
        let username: String = sqlx::query(
            r#"
            UPDATE external_account
            SET public_key = $1,
            private_key = $2
            WHERE username = $3
            RETURNING username
        "#,
        )
        .bind(public_key)
        .bind(private_key)
        .bind(username)
        .fetch_one(&mut *self.connection)
        .await
        .map_err(DatabaseError::SqlError)?
        .get(0);

        self.get(&username).await.map(|x| x.unwrap())
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_conf::Config;

    use crate::utils::use_temporary_db;

    use super::{ExternalAccount, ExternalAccountDB, ExternalAccountDBImpl};

    #[actix_rt::test]
    async fn test_db() {
        use_temporary_db(
            Config::from_env(),
            "external-account-test-db",
            |_config, conn| async move {
                let mut conn = conn.acquire().await?;
                let mut db = ExternalAccountDBImpl::new(&mut conn);

                let _account = db
                    .create(
                        ExternalAccount::builder()
                            .username("me".into())
                            .public_key("sample".into())
                            .private_key("sample".into())
                            .build()?,
                    )
                    .await?;
                assert!(db.get("me").await?.is_some());

                let account = db.set_keys("me", "one", "two").await?;
                assert_eq!(account.public_key(), "one");
                assert_eq!(account.private_key(), "two");

                Ok(())
            },
        )
        .await;
    }
}
