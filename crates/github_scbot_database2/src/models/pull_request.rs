use crate::errors::{ConnectionSnafu, SqlSnafu, TransactionSnafu};
use async_trait::async_trait;
use github_scbot_database_macros::SCGetter;
use github_scbot_types::{pulls::GhMergeStrategy, status::QaStatus};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use sqlx::{postgres::PgRow, FromRow, PgConnection, PgPool, Postgres, Row, Transaction};

use crate::{
    errors::Result,
    fields::{GhMergeStrategyDecode, QaStatusDecode},
    Repository,
};

#[derive(SCGetter, Debug, Clone, Default, derive_builder::Builder, Serialize, Deserialize)]
#[builder(default, setter(into))]
pub struct PullRequest {
    #[get]
    id: u64,
    #[get]
    repository_id: u64,
    #[get]
    number: u64,
    #[get_ref]
    qa_status: QaStatus,
    #[get]
    needed_reviewers_count: u64,
    #[get]
    status_comment_id: u64,
    #[get]
    checks_enabled: bool,
    #[get]
    automerge: bool,
    #[get]
    locked: bool,
    #[get_ref]
    strategy_override: Option<GhMergeStrategy>,
}

impl PullRequest {
    pub fn builder() -> PullRequestBuilder {
        PullRequestBuilder::default()
    }

    pub fn set_repository_id(&mut self, id: u64) {
        self.repository_id = id;
    }
}

impl PullRequestBuilder {
    pub fn with_repository(&mut self, repository: &Repository) -> &mut Self {
        self.repository_id = Some(repository.id());
        self.automerge = Some(repository.default_automerge());
        self.checks_enabled = Some(repository.default_enable_checks());
        self.needed_reviewers_count = Some(repository.default_needed_reviewers_count());
        self.qa_status = if repository.default_enable_qa() {
            None
        } else {
            Some(QaStatus::Skipped)
        };
        self
    }
}

impl<'r> FromRow<'r, PgRow> for PullRequest {
    fn from_row(row: &'r PgRow) -> core::result::Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.try_get::<i32, _>("id")? as u64,
            repository_id: row.try_get::<i32, _>("repository_id")? as u64,
            number: row.try_get::<i32, _>("number")? as u64,
            qa_status: *row.try_get::<QaStatusDecode, _>("qa_status")?,
            needed_reviewers_count: row.try_get::<i32, _>("needed_reviewers_count")? as u64,
            status_comment_id: row.try_get::<i32, _>("status_comment_id")? as u64,
            checks_enabled: row.try_get("checks_enabled")?,
            automerge: row.try_get("automerge")?,
            locked: row.try_get("locked")?,
            strategy_override: row
                .try_get::<Option<GhMergeStrategyDecode>, _>("strategy_override")?
                .map(Into::into),
        })
    }
}

#[async_trait]
#[mockall::automock]
pub trait PullRequestDB {
    async fn create(&mut self, instance: PullRequest) -> Result<PullRequest>;
    async fn update(&mut self, instance: PullRequest) -> Result<PullRequest>;
    async fn get(&mut self, owner: &str, name: &str, number: u64) -> Result<Option<PullRequest>>;
    async fn delete(&mut self, owner: &str, name: &str, number: u64) -> Result<bool>;
    async fn list(&mut self, owner: &str, name: &str) -> Result<Vec<PullRequest>>;
    async fn all(&mut self) -> Result<Vec<PullRequest>>;
    async fn set_qa_status(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        status: QaStatus,
    ) -> Result<PullRequest>;
    async fn set_needed_reviewers_count(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        count: u64,
    ) -> Result<PullRequest>;
    async fn set_status_comment_id(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        id: u64,
    ) -> Result<PullRequest>;
    async fn set_checks_enabled(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        value: bool,
    ) -> Result<PullRequest>;
    async fn set_automerge(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        value: bool,
    ) -> Result<PullRequest>;
    async fn set_locked(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        value: bool,
    ) -> Result<PullRequest>;
    async fn set_strategy_override(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        strategy: Option<GhMergeStrategy>,
    ) -> Result<PullRequest>;
}

pub struct PullRequestDBImpl<'a> {
    connection: &'a mut PgConnection,
}

impl<'a> PullRequestDBImpl<'a> {
    pub fn new(connection: &'a mut PgConnection) -> Self {
        Self { connection }
    }

    async fn get_from_id(&mut self, id: u64) -> Result<Option<PullRequest>> {
        sqlx::query_as::<_, PullRequest>(
            r#"
            SELECT *
            FROM pull_request
            WHERE id = $1
        "#,
        )
        .bind(id as i32)
        .fetch_optional(&mut *self.connection)
        .await
        .context(SqlSnafu)
    }
}

pub struct PullRequestDBImplPool {
    pool: PgPool,
}

impl PullRequestDBImplPool {
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
impl PullRequestDB for PullRequestDBImplPool {
    async fn create(&mut self, instance: PullRequest) -> Result<PullRequest> {
        let mut transaction = self.begin().await?;
        let data = PullRequestDBImpl::new(&mut *transaction)
            .create(instance)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn update(&mut self, instance: PullRequest) -> Result<PullRequest> {
        let mut transaction = self.begin().await?;
        let data = PullRequestDBImpl::new(&mut *transaction)
            .update(instance)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn get(&mut self, owner: &str, name: &str, number: u64) -> Result<Option<PullRequest>> {
        let mut transaction = self.begin().await?;
        let data = PullRequestDBImpl::new(&mut *transaction)
            .get(owner, name, number)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn delete(&mut self, owner: &str, name: &str, number: u64) -> Result<bool> {
        let mut transaction = self.begin().await?;
        let data = PullRequestDBImpl::new(&mut *transaction)
            .delete(owner, name, number)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn list(&mut self, owner: &str, name: &str) -> Result<Vec<PullRequest>> {
        let mut transaction = self.begin().await?;
        let data = PullRequestDBImpl::new(&mut *transaction)
            .list(owner, name)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn all(&mut self) -> Result<Vec<PullRequest>> {
        let mut transaction = self.begin().await?;
        let data = PullRequestDBImpl::new(&mut *transaction).all().await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn set_qa_status(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        status: QaStatus,
    ) -> Result<PullRequest> {
        let mut transaction = self.begin().await?;
        let data = PullRequestDBImpl::new(&mut *transaction)
            .set_qa_status(owner, name, number, status)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn set_needed_reviewers_count(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        count: u64,
    ) -> Result<PullRequest> {
        let mut transaction = self.begin().await?;
        let data = PullRequestDBImpl::new(&mut *transaction)
            .set_needed_reviewers_count(owner, name, number, count)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn set_status_comment_id(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        id: u64,
    ) -> Result<PullRequest> {
        let mut transaction = self.begin().await?;
        let data = PullRequestDBImpl::new(&mut *transaction)
            .set_status_comment_id(owner, name, number, id)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn set_checks_enabled(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        value: bool,
    ) -> Result<PullRequest> {
        let mut transaction = self.begin().await?;
        let data = PullRequestDBImpl::new(&mut *transaction)
            .set_checks_enabled(owner, name, number, value)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn set_automerge(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        value: bool,
    ) -> Result<PullRequest> {
        let mut transaction = self.begin().await?;
        let data = PullRequestDBImpl::new(&mut *transaction)
            .set_automerge(owner, name, number, value)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn set_locked(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        value: bool,
    ) -> Result<PullRequest> {
        let mut transaction = self.begin().await?;
        let data = PullRequestDBImpl::new(&mut *transaction)
            .set_locked(owner, name, number, value)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn set_strategy_override(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        strategy: Option<GhMergeStrategy>,
    ) -> Result<PullRequest> {
        let mut transaction = self.begin().await?;
        let data = PullRequestDBImpl::new(&mut *transaction)
            .set_strategy_override(owner, name, number, strategy)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }
}

#[async_trait]
impl<'a> PullRequestDB for PullRequestDBImpl<'a> {
    #[tracing::instrument(skip(self))]
    async fn create(&mut self, instance: PullRequest) -> Result<PullRequest> {
        let new_id: i32 = sqlx::query(
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
            RETURNING id;
            "#,
        )
        .bind(instance.repository_id as i32)
        .bind(instance.number as i32)
        .bind(instance.qa_status.to_string())
        .bind(instance.needed_reviewers_count as i32)
        .bind(instance.status_comment_id as i32)
        .bind(instance.checks_enabled)
        .bind(instance.automerge)
        .bind(instance.locked)
        .bind(instance.strategy_override.map(|x| x.to_string()))
        .fetch_one(&mut *self.connection)
        .await
        .context(SqlSnafu)?
        .get(0);

        self.get_from_id(new_id as u64).await.map(|x| x.unwrap())
    }

    #[tracing::instrument(skip(self))]
    async fn update(&mut self, instance: PullRequest) -> Result<PullRequest> {
        let new_id: i32 = sqlx::query(
            r#"
            UPDATE pull_request
            SET qa_status = $1,
            needed_reviewers_count = $2,
            status_comment_id = $3,
            checks_enabled = $4,
            automerge = $5,
            locked = $6,
            strategy_override = $7
            WHERE repository_id = $8
            AND number = $9
            RETURNING id;
            "#,
        )
        .bind(instance.qa_status.to_string())
        .bind(instance.needed_reviewers_count as i32)
        .bind(instance.status_comment_id as i32)
        .bind(instance.checks_enabled)
        .bind(instance.automerge)
        .bind(instance.locked)
        .bind(instance.strategy_override.map(|x| x.to_string()))
        .bind(instance.repository_id as i32)
        .bind(instance.number as i32)
        .fetch_one(&mut *self.connection)
        .await
        .context(SqlSnafu)?
        .get(0);

        self.get_from_id(new_id as u64).await.map(|x| x.unwrap())
    }

    #[tracing::instrument(skip(self), ret)]
    async fn get(&mut self, owner: &str, name: &str, number: u64) -> Result<Option<PullRequest>> {
        sqlx::query_as::<_, PullRequest>(
            r#"
            SELECT pull_request.*
            FROM pull_request
            INNER JOIN repository ON (repository_id = repository.id)
            WHERE repository.owner = $1
            AND repository.name = $2
            AND number = $3;
            "#,
        )
        .bind(owner)
        .bind(name)
        .bind(number as i32)
        .fetch_optional(&mut *self.connection)
        .await
        .context(SqlSnafu)
    }

    #[tracing::instrument(skip(self))]
    async fn delete(&mut self, owner: &str, name: &str, number: u64) -> Result<bool> {
        sqlx::query(
            r#"
            DELETE FROM pull_request
            USING repository
            WHERE repository_id = repository.id
            AND repository.owner = $1
            AND repository.name = $2
            AND number = $3;
            "#,
        )
        .bind(owner)
        .bind(name)
        .bind(number as i32)
        .execute(&mut *self.connection)
        .await
        .map(|x| x.rows_affected() > 0)
        .context(SqlSnafu)
    }

    #[tracing::instrument(skip(self))]
    async fn list(&mut self, owner: &str, name: &str) -> Result<Vec<PullRequest>> {
        sqlx::query_as::<_, PullRequest>(
            r#"
            SELECT pull_request.*
            FROM pull_request
            INNER JOIN repository ON (repository_id = repository.id)
            WHERE repository.owner = $1
            AND repository.name = $2;
            "#,
        )
        .bind(owner)
        .bind(name)
        .fetch_all(&mut *self.connection)
        .await
        .context(SqlSnafu)
    }

    #[tracing::instrument(skip(self))]
    async fn all(&mut self) -> Result<Vec<PullRequest>> {
        sqlx::query_as::<_, PullRequest>(
            r#"
            SELECT *
            FROM pull_request;
            "#,
        )
        .fetch_all(&mut *self.connection)
        .await
        .context(SqlSnafu)
    }

    #[tracing::instrument(skip(self))]
    async fn set_qa_status(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        status: QaStatus,
    ) -> Result<PullRequest> {
        let new_id: i32 = sqlx::query(
            r#"
            UPDATE pull_request
            SET qa_status = $1
            FROM repository
            WHERE repository_id = repository.id
            AND repository.owner = $2
            AND repository.name = $3
            AND number = $4
            RETURNING pull_request.id;
            "#,
        )
        .bind(status.to_string())
        .bind(owner)
        .bind(name)
        .bind(number as i32)
        .fetch_one(&mut *self.connection)
        .await
        .context(SqlSnafu)?
        .get(0);

        self.get_from_id(new_id as u64).await.map(|x| x.unwrap())
    }

    #[tracing::instrument(skip(self))]
    async fn set_needed_reviewers_count(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        count: u64,
    ) -> Result<PullRequest> {
        let new_id: i32 = sqlx::query(
            r#"
            UPDATE pull_request
            SET needed_reviewers_count = $1
            FROM repository
            WHERE repository_id = repository.id
            AND repository.owner = $2
            AND repository.name = $3
            AND number = $4
            RETURNING pull_request.id;
            "#,
        )
        .bind(count as i32)
        .bind(owner)
        .bind(name)
        .bind(number as i32)
        .fetch_one(&mut *self.connection)
        .await
        .context(SqlSnafu)?
        .get(0);

        self.get_from_id(new_id as u64).await.map(|x| x.unwrap())
    }

    #[tracing::instrument(skip(self))]
    async fn set_status_comment_id(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        id: u64,
    ) -> Result<PullRequest> {
        let new_id: i32 = sqlx::query(
            r#"
            UPDATE pull_request
            SET status_comment_id = $1
            FROM repository
            WHERE repository_id = repository.id
            AND repository.owner = $2
            AND repository.name = $3
            AND number = $4
            RETURNING pull_request.id;
            "#,
        )
        .bind(id as i32)
        .bind(owner)
        .bind(name)
        .bind(number as i32)
        .fetch_one(&mut *self.connection)
        .await
        .context(SqlSnafu)?
        .get(0);

        self.get_from_id(new_id as u64).await.map(|x| x.unwrap())
    }

    #[tracing::instrument(skip(self))]
    async fn set_checks_enabled(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        value: bool,
    ) -> Result<PullRequest> {
        let new_id: i32 = sqlx::query(
            r#"
            UPDATE pull_request
            SET checks_enabled = $1
            FROM repository
            WHERE repository_id = repository.id
            AND repository.owner = $2
            AND repository.name = $3
            AND number = $4
            RETURNING pull_request.id;
            "#,
        )
        .bind(value)
        .bind(owner)
        .bind(name)
        .bind(number as i32)
        .fetch_one(&mut *self.connection)
        .await
        .context(SqlSnafu)?
        .get(0);

        self.get_from_id(new_id as u64).await.map(|x| x.unwrap())
    }

    #[tracing::instrument(skip(self))]
    async fn set_automerge(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        value: bool,
    ) -> Result<PullRequest> {
        let new_id: i32 = sqlx::query(
            r#"
            UPDATE pull_request
            SET automerge = $1
            FROM repository
            WHERE repository_id = repository.id
            AND repository.owner = $2
            AND repository.name = $3
            AND number = $4
            RETURNING pull_request.id;
            "#,
        )
        .bind(value)
        .bind(owner)
        .bind(name)
        .bind(number as i32)
        .fetch_one(&mut *self.connection)
        .await
        .context(SqlSnafu)?
        .get(0);

        self.get_from_id(new_id as u64).await.map(|x| x.unwrap())
    }

    #[tracing::instrument(skip(self))]
    async fn set_locked(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        value: bool,
    ) -> Result<PullRequest> {
        let new_id: i32 = sqlx::query(
            r#"
            UPDATE pull_request
            SET locked = $1
            FROM repository
            WHERE repository_id = repository.id
            AND repository.owner = $2
            AND repository.name = $3
            AND number = $4
            RETURNING pull_request.id;
            "#,
        )
        .bind(value)
        .bind(owner)
        .bind(name)
        .bind(number as i32)
        .fetch_one(&mut *self.connection)
        .await
        .context(SqlSnafu)?
        .get(0);

        self.get_from_id(new_id as u64).await.map(|x| x.unwrap())
    }

    #[tracing::instrument(skip(self))]
    async fn set_strategy_override(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        strategy: Option<GhMergeStrategy>,
    ) -> Result<PullRequest> {
        let new_id: i32 = sqlx::query(
            r#"
            UPDATE pull_request
            SET strategy_override = $1
            FROM repository
            WHERE repository_id = repository.id
            AND repository.owner = $2
            AND repository.name = $3
            AND number = $4
            RETURNING pull_request.id;
            "#,
        )
        .bind(strategy.map(|x| x.to_string()))
        .bind(owner)
        .bind(name)
        .bind(number as i32)
        .fetch_one(&mut *self.connection)
        .await
        .context(SqlSnafu)?
        .get(0);

        self.get_from_id(new_id as u64).await.map(|x| x.unwrap())
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_conf::Config;
    use github_scbot_types::{pulls::GhMergeStrategy, status::QaStatus};

    use crate::{
        models::{
            pull_request::PullRequestDB,
            repository::{Repository, RepositoryDB},
        },
        utils::use_temporary_db,
        RepositoryDBImpl,
    };

    use super::{PullRequest, PullRequestDBImpl};

    #[actix_rt::test]
    async fn test_db() {
        use_temporary_db(
            Config::from_env(),
            "pull-request-test-db",
            |config, conn| async move {
                let mut conn = conn.acquire().await?;
                let mut repo_db = RepositoryDBImpl::new(&mut conn);

                let repo = {
                    repo_db
                        .create(Repository::builder().with_config(&config).build()?)
                        .await?
                };

                let mut db = PullRequestDBImpl::new(&mut conn);

                // Create
                let _pr = db
                    .create(
                        PullRequest::builder()
                            .with_repository(&repo)
                            .number(10u64)
                            .build()?,
                    )
                    .await?;

                // Update
                let _pr = db
                    .update(
                        PullRequest::builder()
                            .with_repository(&repo)
                            .number(10u64)
                            .automerge(true)
                            .build()?,
                    )
                    .await?;

                // Get
                assert!(db.get("", "", 10).await?.is_some());

                // List
                assert_eq!(db.list("", "").await?.len(), 1);

                // All
                assert_eq!(db.all().await?.len(), 1);

                // Automerge
                assert!(db.set_automerge("", "", 10, true).await?.automerge);

                // Checks enabled
                assert!(
                    db.set_checks_enabled("", "", 10, true)
                        .await?
                        .checks_enabled
                );

                // Status comment ID
                assert_eq!(
                    db.set_status_comment_id("", "", 10, 1234)
                        .await?
                        .status_comment_id,
                    1234
                );

                // Reviewers count
                assert_eq!(
                    db.set_needed_reviewers_count("", "", 10, 10)
                        .await?
                        .needed_reviewers_count,
                    10
                );

                // QA status
                assert_eq!(
                    db.set_qa_status("", "", 10, QaStatus::Waiting)
                        .await?
                        .qa_status,
                    QaStatus::Waiting
                );

                // Locked
                assert!(db.set_locked("", "", 10, true).await?.locked);

                // Strategy override
                assert_eq!(
                    db.set_strategy_override("", "", 10, Some(GhMergeStrategy::Squash))
                        .await?
                        .strategy_override,
                    Some(GhMergeStrategy::Squash)
                );
                assert_eq!(
                    db.set_strategy_override("", "", 10, None)
                        .await?
                        .strategy_override,
                    None
                );

                // Delete
                assert!(db.delete("", "", 10).await?, "PR should exist");

                Ok(())
            },
        )
        .await;
    }
}
