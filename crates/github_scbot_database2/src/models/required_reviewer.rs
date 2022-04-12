use async_trait::async_trait;
use github_scbot_database_macros::SCGetter;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, FromRow, PgConnection, PgPool, Postgres, Row, Transaction};

use crate::{errors::Result, DatabaseError};

#[derive(SCGetter, Debug, Clone, Default, derive_builder::Builder, Serialize, Deserialize)]
#[builder(default)]
pub struct RequiredReviewer {
    #[get]
    pull_request_id: u64,
    #[get_deref]
    username: String,
}

impl RequiredReviewer {
    pub fn builder() -> RequiredReviewerBuilder {
        RequiredReviewerBuilder::default()
    }

    pub fn set_pull_request_id(&mut self, id: u64) {
        self.pull_request_id = id;
    }
}

impl<'r> FromRow<'r, PgRow> for RequiredReviewer {
    fn from_row(row: &'r PgRow) -> core::result::Result<Self, sqlx::Error> {
        Ok(Self {
            pull_request_id: row.try_get::<i32, _>("pull_request_id")? as u64,
            username: row.try_get("username")?,
        })
    }
}

#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait RequiredReviewerDB {
    async fn create(&mut self, instance: RequiredReviewer) -> Result<RequiredReviewer>;
    async fn list(&mut self, owner: &str, name: &str, number: u64)
        -> Result<Vec<RequiredReviewer>>;
    async fn get(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        username: &str,
    ) -> Result<Option<RequiredReviewer>>;
    async fn delete(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        username: &str,
    ) -> Result<bool>;
    async fn all(&mut self) -> Result<Vec<RequiredReviewer>>;
}

pub struct RequiredReviewerDBImpl<'a> {
    connection: &'a mut PgConnection,
}

impl<'a> RequiredReviewerDBImpl<'a> {
    pub fn new(connection: &'a mut PgConnection) -> Self {
        Self { connection }
    }

    async fn get_from_pull_request_id(
        &mut self,
        pull_request_id: u64,
        username: &str,
    ) -> Result<Option<RequiredReviewer>> {
        sqlx::query_as::<_, RequiredReviewer>(
            r#"
                SELECT *
                FROM required_reviewer
                WHERE pull_request_id = $1
                AND username = $2
            "#,
        )
        .bind(pull_request_id as i32)
        .bind(username)
        .fetch_optional(&mut *self.connection)
        .await
        .map_err(DatabaseError::SqlError)
    }
}

pub struct RequiredReviewerDBImplPool {
    pool: PgPool,
}

impl RequiredReviewerDBImplPool {
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
impl RequiredReviewerDB for RequiredReviewerDBImplPool {
    async fn create(&mut self, instance: RequiredReviewer) -> Result<RequiredReviewer> {
        let mut transaction = self.begin().await?;
        let data = RequiredReviewerDBImpl::new(&mut *transaction)
            .create(instance)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn get(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        username: &str,
    ) -> Result<Option<RequiredReviewer>> {
        let mut transaction = self.begin().await?;
        let data = RequiredReviewerDBImpl::new(&mut *transaction)
            .get(owner, name, number, username)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn delete(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        username: &str,
    ) -> Result<bool> {
        let mut transaction = self.begin().await?;
        let data = RequiredReviewerDBImpl::new(&mut *transaction)
            .delete(owner, name, number, username)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn list(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
    ) -> Result<Vec<RequiredReviewer>> {
        let mut transaction = self.begin().await?;
        let data = RequiredReviewerDBImpl::new(&mut *transaction)
            .list(owner, name, number)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn all(&mut self) -> Result<Vec<RequiredReviewer>> {
        let mut transaction = self.begin().await?;
        let data = RequiredReviewerDBImpl::new(&mut *transaction).all().await?;
        self.commit(transaction).await?;
        Ok(data)
    }
}

#[async_trait]
impl<'a> RequiredReviewerDB for RequiredReviewerDBImpl<'a> {
    async fn create(&mut self, instance: RequiredReviewer) -> Result<RequiredReviewer> {
        sqlx::query(
            r#"
            INSERT INTO required_reviewer
            (
                pull_request_id,
                username
            )
            VALUES
            (
                $1,
                $2
            );
        "#,
        )
        .bind(instance.pull_request_id as i32)
        .bind(instance.username())
        .execute(&mut *self.connection)
        .await
        .map_err(DatabaseError::SqlError)?;

        self.get_from_pull_request_id(instance.pull_request_id, instance.username())
            .await
            .map(|x| x.unwrap())
    }

    async fn get(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        username: &str,
    ) -> Result<Option<RequiredReviewer>> {
        sqlx::query_as::<_, RequiredReviewer>(
            r#"
                SELECT *
                FROM required_reviewer
                INNER JOIN repository ON (repository.owner = $1 AND repository.name = $2)
                INNER JOIN pull_request ON (pull_request.repository_id = repository.id AND pull_request.number = $3 AND required_reviewer.pull_request_id = pull_request.id)
                WHERE username = $4
            "#
        )
        .bind(owner)
        .bind(name)
        .bind(number as i32)
        .bind(username)
        .fetch_optional(&mut *self.connection)
        .await
        .map_err(DatabaseError::SqlError)
    }

    async fn delete(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        username: &str,
    ) -> Result<bool> {
        sqlx::query(
            r#"
            DELETE FROM required_reviewer
            USING repository, pull_request
            WHERE repository.owner = $1
            AND repository.name = $2
            AND pull_request.repository_id = repository.id
            AND pull_request.number = $3
            AND required_reviewer.pull_request_id = pull_request.id
            AND username = $4
        "#,
        )
        .bind(owner)
        .bind(name)
        .bind(number as i32)
        .bind(username)
        .execute(&mut *self.connection)
        .await
        .map(|x| x.rows_affected() > 0)
        .map_err(DatabaseError::SqlError)
    }

    async fn list(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
    ) -> Result<Vec<RequiredReviewer>> {
        sqlx::query_as::<_, RequiredReviewer>(
            r#"
                SELECT *
                FROM required_reviewer
                INNER JOIN repository ON (repository.owner = $1 AND repository.name = $2)
                INNER JOIN pull_request ON (pull_request.repository_id = repository.id AND pull_request.number = $3 AND required_reviewer.pull_request_id = pull_request.id)
            "#
        )
        .bind(owner)
        .bind(name)
        .bind(number as i32)
        .fetch_all(&mut *self.connection)
        .await
        .map_err(DatabaseError::SqlError)
    }

    async fn all(&mut self) -> Result<Vec<RequiredReviewer>> {
        sqlx::query_as::<_, RequiredReviewer>(
            r#"
                SELECT *
                FROM required_reviewer
            "#,
        )
        .fetch_all(&mut *self.connection)
        .await
        .map_err(DatabaseError::SqlError)
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_conf::Config;

    use crate::{
        models::repository::{Repository, RepositoryDB},
        utils::use_temporary_db,
        RepositoryDBImpl,
    };

    use super::{RequiredReviewer, RequiredReviewerDB, RequiredReviewerDBImpl};

    #[actix_rt::test]
    async fn test_db() {
        use_temporary_db(
            Config::from_env(),
            "required-reviewer-test-db",
            |_config, conn| async move {
                let mut conn = conn.acquire().await?;
                let repo = {
                    let mut db = RepositoryDBImpl::new(&mut conn);
                    db.create(Repository::builder().build()?).await?
                };

                sqlx::query(
                    r#"
                INSERT INTO pull_request
                (
                    repository_id,
                    number,
                    qa_status,
                    needed_reviewers_count,
                    status_comment_id,
                    checks_enabled,
                    automerge,
                    locked,
                    strategy_override
                )
                VALUES
                (
                    $1,
                    $2,
                    $3,
                    $4,
                    $5,
                    $6,
                    $7,
                    $8,
                    $9
                )
            "#,
                )
                .bind(repo.id() as i32)
                .bind(1i32)
                .bind("skipped")
                .bind(0i32)
                .bind(0i32)
                .bind(false)
                .bind(false)
                .bind(false)
                .bind("")
                .execute(&mut conn)
                .await?;

                let mut db = RequiredReviewerDBImpl::new(&mut conn);
                let _reviewer = db
                    .create(
                        RequiredReviewer::builder()
                            .pull_request_id(1)
                            .username("test".into())
                            .build()?,
                    )
                    .await?;
                let _reviewer = db
                    .create(
                        RequiredReviewer::builder()
                            .pull_request_id(1)
                            .username("test2".into())
                            .build()?,
                    )
                    .await?;

                assert!(
                    db.get("", "", 1, "nope").await?.is_none(),
                    "the 'nope' user is not a required reviewer"
                );
                assert!(
                    db.get("", "", 1, "test").await?.is_some(),
                    "the 'test' user is a required reviewer"
                );
                assert_eq!(
                    db.list("", "", 1).await?.len(),
                    2,
                    "there should be two required reviewers"
                );
                assert!(
                    db.delete("", "", 1, "test").await?,
                    "the 'test' user deletion should work"
                );

                Ok(())
            },
        )
        .await;
    }
}
