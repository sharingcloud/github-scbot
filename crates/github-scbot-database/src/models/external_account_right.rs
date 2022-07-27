use async_trait::async_trait;
use github_scbot_macros::SCGetter;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use sqlx::{postgres::PgRow, FromRow, PgConnection, PgPool, Postgres, Row, Transaction};

use crate::errors::{ConnectionSnafu, SqlSnafu, TransactionSnafu};
use crate::{errors::Result, Repository};

#[derive(SCGetter, Debug, Clone, Default, derive_builder::Builder, Serialize, Deserialize)]
#[builder(default, setter(into))]
pub struct ExternalAccountRight {
    #[get_deref]
    username: String,
    #[get]
    repository_id: u64,
}

impl ExternalAccountRight {
    pub fn builder() -> ExternalAccountRightBuilder {
        ExternalAccountRightBuilder::default()
    }

    pub fn set_repository_id(&mut self, id: u64) {
        self.repository_id = id;
    }
}

impl ExternalAccountRightBuilder {
    pub fn with_repository(&mut self, repository: &Repository) -> &mut Self {
        self.repository_id = Some(repository.id());
        self
    }
}

impl<'r> FromRow<'r, PgRow> for ExternalAccountRight {
    fn from_row(row: &'r PgRow) -> core::result::Result<Self, sqlx::Error> {
        Ok(Self {
            username: row.try_get("username")?,
            repository_id: row.try_get::<i32, _>("repository_id")? as u64,
        })
    }
}

#[async_trait]
#[mockall::automock]
pub trait ExternalAccountRightDB {
    async fn create(&mut self, instance: ExternalAccountRight) -> Result<ExternalAccountRight>;
    async fn get(
        &mut self,
        owner: &str,
        name: &str,
        username: &str,
    ) -> Result<Option<ExternalAccountRight>>;
    async fn delete(&mut self, owner: &str, name: &str, username: &str) -> Result<bool>;
    async fn delete_all(&mut self, username: &str) -> Result<bool>;
    async fn list(&mut self, username: &str) -> Result<Vec<ExternalAccountRight>>;
    async fn all(&mut self) -> Result<Vec<ExternalAccountRight>>;
}

pub struct ExternalAccountRightDBImpl<'a> {
    connection: &'a mut PgConnection,
}

impl<'a> ExternalAccountRightDBImpl<'a> {
    pub fn new(connection: &'a mut PgConnection) -> Self {
        Self { connection }
    }

    async fn get_from_id(
        &mut self,
        username: &str,
        repository_id: u64,
    ) -> Result<Option<ExternalAccountRight>> {
        sqlx::query_as::<_, ExternalAccountRight>(
            r#"
            SELECT *
            FROM external_account_right
            WHERE username = $1
            AND repository_id = $2
        "#,
        )
        .bind(username)
        .bind(repository_id as i32)
        .fetch_optional(&mut *self.connection)
        .await
        .context(SqlSnafu)
    }
}

pub struct ExternalAccountRightDBImplPool {
    pool: PgPool,
}

impl ExternalAccountRightDBImplPool {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn begin<'a>(&mut self) -> Result<Transaction<'a, Postgres>> {
        self.pool.begin().await.context(ConnectionSnafu)
    }

    pub async fn commit<'a>(&mut self, transaction: Transaction<'a, Postgres>) -> Result<()> {
        transaction.commit().await.context(TransactionSnafu)
    }
}

#[async_trait]
impl ExternalAccountRightDB for ExternalAccountRightDBImplPool {
    async fn create(&mut self, instance: ExternalAccountRight) -> Result<ExternalAccountRight> {
        let mut transaction = self.begin().await?;
        let data = ExternalAccountRightDBImpl::new(&mut *transaction)
            .create(instance)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn get(
        &mut self,
        owner: &str,
        name: &str,
        username: &str,
    ) -> Result<Option<ExternalAccountRight>> {
        let mut transaction = self.begin().await?;
        let data = ExternalAccountRightDBImpl::new(&mut *transaction)
            .get(owner, name, username)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn delete(&mut self, owner: &str, name: &str, username: &str) -> Result<bool> {
        let mut transaction = self.begin().await?;
        let data = ExternalAccountRightDBImpl::new(&mut *transaction)
            .delete(owner, name, username)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn delete_all(&mut self, username: &str) -> Result<bool> {
        let mut transaction = self.begin().await?;
        let data = ExternalAccountRightDBImpl::new(&mut *transaction)
            .delete_all(username)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn list(&mut self, username: &str) -> Result<Vec<ExternalAccountRight>> {
        let mut transaction = self.begin().await?;
        let data = ExternalAccountRightDBImpl::new(&mut *transaction)
            .list(username)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn all(&mut self) -> Result<Vec<ExternalAccountRight>> {
        let mut transaction = self.begin().await?;
        let data = ExternalAccountRightDBImpl::new(&mut *transaction)
            .all()
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }
}

#[async_trait]
impl<'a> ExternalAccountRightDB for ExternalAccountRightDBImpl<'a> {
    #[tracing::instrument(skip(self))]
    async fn create(&mut self, instance: ExternalAccountRight) -> Result<ExternalAccountRight> {
        sqlx::query(
            r#"
            INSERT INTO external_account_right
            (
                username,
                repository_id
            ) VALUES (
                $1,
                $2
            )
            RETURNING repository_id;
            "#,
        )
        .bind(&instance.username)
        .bind(instance.repository_id as i32)
        .execute(&mut *self.connection)
        .await
        .context(SqlSnafu)?;

        self.get_from_id(&instance.username, instance.repository_id)
            .await
            .map(|x| x.unwrap())
    }

    #[tracing::instrument(skip(self))]
    async fn get(
        &mut self,
        owner: &str,
        name: &str,
        username: &str,
    ) -> Result<Option<ExternalAccountRight>> {
        sqlx::query_as::<_, ExternalAccountRight>(
            r#"
                SELECT external_account_right.*
                FROM external_account_right
                INNER JOIN repository ON (repository.id = repository_id)
                WHERE repository.owner = $1
                AND repository.name = $2
                AND username = $3
            "#,
        )
        .bind(owner)
        .bind(name)
        .bind(username)
        .fetch_optional(&mut *self.connection)
        .await
        .context(SqlSnafu)
    }

    #[tracing::instrument(skip(self))]
    async fn delete(&mut self, owner: &str, name: &str, username: &str) -> Result<bool> {
        sqlx::query(
            r#"
            DELETE FROM external_account_right
            USING repository
            WHERE repository.id = repository_id
            AND repository.owner = $1
            AND repository.name = $2
            AND username = $3
        "#,
        )
        .bind(owner)
        .bind(name)
        .bind(username)
        .execute(&mut *self.connection)
        .await
        .map(|x| x.rows_affected() > 0)
        .context(SqlSnafu)
    }

    #[tracing::instrument(skip(self))]
    async fn delete_all(&mut self, username: &str) -> Result<bool> {
        sqlx::query(
            r#"
            DELETE FROM external_account_right
            WHERE username = $1
        "#,
        )
        .bind(username)
        .execute(&mut *self.connection)
        .await
        .map(|x| x.rows_affected() > 0)
        .context(SqlSnafu)
    }

    #[tracing::instrument(skip(self))]
    async fn list(&mut self, username: &str) -> Result<Vec<ExternalAccountRight>> {
        sqlx::query_as::<_, ExternalAccountRight>(
            r#"
            SELECT *
            FROM external_account_right
            WHERE username = $1
        "#,
        )
        .bind(username)
        .fetch_all(&mut *self.connection)
        .await
        .context(SqlSnafu)
    }

    #[tracing::instrument(skip(self))]
    async fn all(&mut self) -> Result<Vec<ExternalAccountRight>> {
        sqlx::query_as::<_, ExternalAccountRight>(
            r#"
            SELECT *
            FROM external_account_right
        "#,
        )
        .fetch_all(&mut *self.connection)
        .await
        .context(SqlSnafu)
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_core::config::Config;

    use crate::{
        utils::use_temporary_db, ExternalAccount, ExternalAccountDB, ExternalAccountDBImpl,
        Repository, RepositoryDB, RepositoryDBImpl,
    };

    use super::{ExternalAccountRight, ExternalAccountRightDB, ExternalAccountRightDBImpl};

    #[actix_rt::test]
    async fn test_db() {
        use_temporary_db(
            Config::from_env(),
            "external-account-right-test-db",
            |_config, conn| async move {
                let mut conn = conn.acquire().await?;

                let repo = {
                    let mut db = RepositoryDBImpl::new(&mut conn);
                    db.create(Repository::builder().build()?).await?
                };

                let account = {
                    let mut db = ExternalAccountDBImpl::new(&mut conn);
                    db.create(ExternalAccount::builder().username("me").build()?)
                        .await?
                };

                let mut db = ExternalAccountRightDBImpl::new(&mut conn);
                let _right = db
                    .create(
                        ExternalAccountRight::builder()
                            .username(account.username())
                            .repository_id(repo.id())
                            .build()?,
                    )
                    .await?;

                assert!(db.get("", "", "me").await?.is_some());
                assert!(db.delete("", "", "me").await?);

                Ok(())
            },
        )
        .await;
    }
}
