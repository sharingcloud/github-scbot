use async_trait::async_trait;
use github_scbot_core::config::Config;
use github_scbot_core::types::pulls::GhMergeStrategy;
use github_scbot_macros::SCGetter;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, FromRow, PgConnection, PgPool, Postgres, Row, Transaction};

use crate::fields::GhMergeStrategyDecode;
use crate::{DatabaseError, Result};

#[derive(
    SCGetter, Debug, Clone, derive_builder::Builder, Serialize, Deserialize, PartialEq, Eq,
)]
#[builder(default, setter(into))]
pub struct Repository {
    #[get]
    id: u64,
    #[get_deref]
    owner: String,
    #[get_deref]
    name: String,
    #[get]
    manual_interaction: bool,
    #[get_deref]
    pr_title_validation_regex: String,
    #[get]
    default_strategy: GhMergeStrategy,
    #[get]
    default_needed_reviewers_count: u64,
    #[get]
    default_automerge: bool,
    #[get]
    default_enable_qa: bool,
    #[get]
    default_enable_checks: bool,
}

impl Default for Repository {
    fn default() -> Self {
        Self {
            id: 0,
            owner: String::new(),
            name: String::new(),
            manual_interaction: false,
            pr_title_validation_regex: String::new(),
            default_strategy: GhMergeStrategy::Merge,
            default_needed_reviewers_count: 0,
            default_automerge: false,
            default_enable_qa: false,
            default_enable_checks: true,
        }
    }
}

impl Repository {
    pub fn builder() -> RepositoryBuilder {
        RepositoryBuilder::default()
    }

    pub fn path(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }
}

impl RepositoryBuilder {
    pub fn with_config(&mut self, config: &Config) -> &mut Self {
        self.default_strategy = (&config.default_merge_strategy).try_into().ok();
        self.default_needed_reviewers_count = Some(config.default_needed_reviewers_count);
        self.pr_title_validation_regex = Some(config.default_pr_title_validation_regex.clone());
        self
    }
}

impl<'r> FromRow<'r, PgRow> for Repository {
    fn from_row(row: &'r PgRow) -> core::result::Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.try_get::<i32, _>("id")? as u64,
            owner: row.try_get("owner")?,
            name: row.try_get("name")?,
            manual_interaction: row.try_get("manual_interaction")?,
            pr_title_validation_regex: row.try_get("pr_title_validation_regex")?,
            default_strategy: *row.try_get::<GhMergeStrategyDecode, _>("default_strategy")?,
            default_needed_reviewers_count: row
                .try_get::<i32, _>("default_needed_reviewers_count")?
                as u64,
            default_automerge: row.try_get("default_automerge")?,
            default_enable_qa: row.try_get("default_enable_qa")?,
            default_enable_checks: row.try_get("default_enable_checks")?,
        })
    }
}

#[async_trait]
#[mockall::automock]
pub trait RepositoryDB {
    async fn create(&mut self, instance: Repository) -> Result<Repository>;
    async fn update(&mut self, instance: Repository) -> Result<Repository>;
    async fn all(&mut self) -> Result<Vec<Repository>>;
    async fn get(&mut self, owner: &str, name: &str) -> Result<Option<Repository>>;
    async fn get_from_id(&mut self, id: u64) -> Result<Option<Repository>>;
    async fn delete(&mut self, owner: &str, name: &str) -> Result<bool>;
    async fn set_manual_interaction(
        &mut self,
        owner: &str,
        name: &str,
        value: bool,
    ) -> Result<Repository>;
    async fn set_pr_title_validation_regex(
        &mut self,
        owner: &str,
        name: &str,
        value: &str,
    ) -> Result<Repository>;
    async fn set_default_strategy(
        &mut self,
        owner: &str,
        name: &str,
        strategy: GhMergeStrategy,
    ) -> Result<Repository>;
    async fn set_default_needed_reviewers_count(
        &mut self,
        owner: &str,
        name: &str,
        count: u64,
    ) -> Result<Repository>;
    async fn set_default_automerge(
        &mut self,
        owner: &str,
        name: &str,
        value: bool,
    ) -> Result<Repository>;
    async fn set_default_enable_qa(
        &mut self,
        owner: &str,
        name: &str,
        value: bool,
    ) -> Result<Repository>;
    async fn set_default_enable_checks(
        &mut self,
        owner: &str,
        name: &str,
        value: bool,
    ) -> Result<Repository>;
}

pub struct RepositoryDBImpl<'a> {
    connection: &'a mut PgConnection,
}

impl<'a> RepositoryDBImpl<'a> {
    pub fn new(connection: &'a mut PgConnection) -> Self {
        Self { connection }
    }
}

pub struct RepositoryDBImplPool {
    pool: PgPool,
}

impl RepositoryDBImplPool {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn begin<'a>(&mut self) -> Result<Transaction<'a, Postgres>> {
        self.pool
            .begin()
            .await
            .map_err(|e| DatabaseError::ConnectionError { source: e })
    }

    pub async fn commit<'a>(&mut self, transaction: Transaction<'a, Postgres>) -> Result<()> {
        transaction
            .commit()
            .await
            .map_err(|e| DatabaseError::TransactionError { source: e })
    }
}

#[async_trait]
impl RepositoryDB for RepositoryDBImplPool {
    async fn create(&mut self, instance: Repository) -> Result<Repository> {
        let mut transaction = self.begin().await?;
        let data = RepositoryDBImpl::new(&mut *transaction)
            .create(instance)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn update(&mut self, instance: Repository) -> Result<Repository> {
        let mut transaction = self.begin().await?;
        let data = RepositoryDBImpl::new(&mut *transaction)
            .update(instance)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn all(&mut self) -> Result<Vec<Repository>> {
        let mut transaction = self.begin().await?;
        let data = RepositoryDBImpl::new(&mut *transaction).all().await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn get_from_id(&mut self, id: u64) -> Result<Option<Repository>> {
        let mut transaction = self.begin().await?;
        let data = RepositoryDBImpl::new(&mut *transaction)
            .get_from_id(id)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn get(&mut self, owner: &str, name: &str) -> Result<Option<Repository>> {
        let mut transaction = self.begin().await?;
        let data = RepositoryDBImpl::new(&mut *transaction)
            .get(owner, name)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn delete(&mut self, owner: &str, name: &str) -> Result<bool> {
        let mut transaction = self.begin().await?;
        let data = RepositoryDBImpl::new(&mut *transaction)
            .delete(owner, name)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn set_manual_interaction(
        &mut self,
        owner: &str,
        name: &str,
        value: bool,
    ) -> Result<Repository> {
        let mut transaction = self.begin().await?;
        let data = RepositoryDBImpl::new(&mut *transaction)
            .set_manual_interaction(owner, name, value)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn set_pr_title_validation_regex(
        &mut self,
        owner: &str,
        name: &str,
        value: &str,
    ) -> Result<Repository> {
        let mut transaction = self.begin().await?;
        let data = RepositoryDBImpl::new(&mut *transaction)
            .set_pr_title_validation_regex(owner, name, value)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn set_default_strategy(
        &mut self,
        owner: &str,
        name: &str,
        strategy: GhMergeStrategy,
    ) -> Result<Repository> {
        let mut transaction = self.begin().await?;
        let data = RepositoryDBImpl::new(&mut *transaction)
            .set_default_strategy(owner, name, strategy)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn set_default_automerge(
        &mut self,
        owner: &str,
        name: &str,
        value: bool,
    ) -> Result<Repository> {
        let mut transaction = self.begin().await?;
        let data = RepositoryDBImpl::new(&mut *transaction)
            .set_default_automerge(owner, name, value)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn set_default_needed_reviewers_count(
        &mut self,
        owner: &str,
        name: &str,
        value: u64,
    ) -> Result<Repository> {
        let mut transaction = self.begin().await?;
        let data = RepositoryDBImpl::new(&mut *transaction)
            .set_default_needed_reviewers_count(owner, name, value)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn set_default_enable_qa(
        &mut self,
        owner: &str,
        name: &str,
        value: bool,
    ) -> Result<Repository> {
        let mut transaction = self.begin().await?;
        let data = RepositoryDBImpl::new(&mut *transaction)
            .set_default_enable_qa(owner, name, value)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn set_default_enable_checks(
        &mut self,
        owner: &str,
        name: &str,
        value: bool,
    ) -> Result<Repository> {
        let mut transaction = self.begin().await?;
        let data = RepositoryDBImpl::new(&mut *transaction)
            .set_default_enable_checks(owner, name, value)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }
}

#[async_trait]
impl<'a> RepositoryDB for RepositoryDBImpl<'a> {
    #[tracing::instrument(skip(self))]
    async fn create(&mut self, instance: Repository) -> Result<Repository> {
        let new_id: i32 = sqlx::query(
            r#"
            INSERT INTO repository
            (
                name,
                owner,
                manual_interaction,
                pr_title_validation_regex,
                default_strategy,
                default_needed_reviewers_count,
                default_automerge,
                default_enable_qa,
                default_enable_checks
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
            RETURNING id
            ;
        "#,
        )
        .bind(instance.name)
        .bind(instance.owner)
        .bind(instance.manual_interaction)
        .bind(instance.pr_title_validation_regex)
        .bind(instance.default_strategy.to_string())
        .bind(instance.default_needed_reviewers_count as i32)
        .bind(instance.default_automerge)
        .bind(instance.default_enable_qa)
        .bind(instance.default_enable_checks)
        .fetch_one(&mut *self.connection)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })?
        .get(0);

        self.get_from_id(new_id as u64).await.map(|x| x.unwrap())
    }

    #[tracing::instrument(skip(self))]
    async fn update(&mut self, instance: Repository) -> Result<Repository> {
        let new_id: i32 = sqlx::query(
            r#"
            UPDATE repository
            SET manual_interaction = $1,
            pr_title_validation_regex = $2,
            default_strategy = $3,
            default_needed_reviewers_count = $4,
            default_automerge = $5,
            default_enable_qa = $6,
            default_enable_checks = $7
            WHERE owner = $8
            AND name = $9
            RETURNING id
            ;
        "#,
        )
        .bind(instance.manual_interaction)
        .bind(instance.pr_title_validation_regex)
        .bind(instance.default_strategy.to_string())
        .bind(instance.default_needed_reviewers_count as i32)
        .bind(instance.default_automerge)
        .bind(instance.default_enable_qa)
        .bind(instance.default_enable_checks)
        .bind(instance.owner)
        .bind(instance.name)
        .fetch_one(&mut *self.connection)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })?
        .get(0);

        self.get_from_id(new_id as u64).await.map(|x| x.unwrap())
    }

    #[tracing::instrument(skip(self))]
    async fn all(&mut self) -> Result<Vec<Repository>> {
        sqlx::query_as::<_, Repository>(
            r#"
                SELECT *
                FROM repository;
            "#,
        )
        .fetch_all(&mut *self.connection)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })
    }

    #[tracing::instrument(skip(self))]
    async fn get_from_id(&mut self, id: u64) -> Result<Option<Repository>> {
        sqlx::query_as::<_, Repository>(
            r#"
            SELECT *
            FROM repository
            WHERE id = $1
        "#,
        )
        .bind(id as i32)
        .fetch_optional(&mut *self.connection)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })
    }

    #[tracing::instrument(skip(self))]
    async fn get(&mut self, owner: &str, name: &str) -> Result<Option<Repository>> {
        sqlx::query_as::<_, Repository>(
            r#"
            SELECT *
            FROM repository
            WHERE owner = $1
            AND name = $2
        "#,
        )
        .bind(owner)
        .bind(name)
        .fetch_optional(&mut *self.connection)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })
    }

    #[tracing::instrument(skip(self))]
    async fn delete(&mut self, owner: &str, name: &str) -> Result<bool> {
        sqlx::query(
            r#"
            DELETE FROM repository
            WHERE owner = $1 AND name = $2
        "#,
        )
        .bind(owner)
        .bind(name)
        .execute(&mut *self.connection)
        .await
        .map(|x| x.rows_affected() > 0)
        .map_err(|e| DatabaseError::SqlError { source: e })
    }

    #[tracing::instrument(skip(self))]
    async fn set_manual_interaction(
        &mut self,
        owner: &str,
        name: &str,
        value: bool,
    ) -> Result<Repository> {
        let id: i32 = sqlx::query(
            r#"
            UPDATE repository
            SET manual_interaction = $1
            WHERE owner = $2
            AND name = $3
            RETURNING id
        "#,
        )
        .bind(value)
        .bind(owner)
        .bind(name)
        .fetch_one(&mut *self.connection)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })?
        .get(0);

        self.get_from_id(id as u64).await.map(|x| x.unwrap())
    }

    #[tracing::instrument(skip(self))]
    async fn set_pr_title_validation_regex(
        &mut self,
        owner: &str,
        name: &str,
        value: &str,
    ) -> Result<Repository> {
        let id: i32 = sqlx::query(
            r#"
            UPDATE repository
            SET pr_title_validation_regex = $1
            WHERE owner = $2
            AND name = $3
            RETURNING id
        "#,
        )
        .bind(value)
        .bind(owner)
        .bind(name)
        .fetch_one(&mut *self.connection)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })?
        .get(0);

        self.get_from_id(id as u64).await.map(|x| x.unwrap())
    }

    #[tracing::instrument(skip(self))]
    async fn set_default_strategy(
        &mut self,
        owner: &str,
        name: &str,
        strategy: GhMergeStrategy,
    ) -> Result<Repository> {
        let id: i32 = sqlx::query(
            r#"
            UPDATE repository
            SET default_strategy = $1
            WHERE owner = $2
            AND name = $3
            RETURNING id
        "#,
        )
        .bind(strategy.to_string())
        .bind(owner)
        .bind(name)
        .fetch_one(&mut *self.connection)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })?
        .get(0);

        self.get_from_id(id as u64).await.map(|x| x.unwrap())
    }

    #[tracing::instrument(skip(self))]
    async fn set_default_needed_reviewers_count(
        &mut self,
        owner: &str,
        name: &str,
        count: u64,
    ) -> Result<Repository> {
        let id: i32 = sqlx::query(
            r#"
            UPDATE repository
            SET default_needed_reviewers_count = $1
            WHERE owner = $2
            AND name = $3
            RETURNING id
        "#,
        )
        .bind(count as i32)
        .bind(owner)
        .bind(name)
        .fetch_one(&mut *self.connection)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })?
        .get(0);

        self.get_from_id(id as u64).await.map(|x| x.unwrap())
    }

    #[tracing::instrument(skip(self))]
    async fn set_default_automerge(
        &mut self,
        owner: &str,
        name: &str,
        value: bool,
    ) -> Result<Repository> {
        let id: i32 = sqlx::query(
            r#"
            UPDATE repository
            SET default_automerge = $1
            WHERE owner = $2
            AND name = $3
            RETURNING id
        "#,
        )
        .bind(value)
        .bind(owner)
        .bind(name)
        .fetch_one(&mut *self.connection)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })?
        .get(0);

        self.get_from_id(id as u64).await.map(|x| x.unwrap())
    }

    #[tracing::instrument(skip(self))]
    async fn set_default_enable_qa(
        &mut self,
        owner: &str,
        name: &str,
        value: bool,
    ) -> Result<Repository> {
        let id: i32 = sqlx::query(
            r#"
            UPDATE repository
            SET default_enable_qa = $1
            WHERE owner = $2
            AND name = $3
            RETURNING id
        "#,
        )
        .bind(value)
        .bind(owner)
        .bind(name)
        .fetch_one(&mut *self.connection)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })?
        .get(0);

        self.get_from_id(id as u64).await.map(|x| x.unwrap())
    }

    #[tracing::instrument(skip(self))]
    async fn set_default_enable_checks(
        &mut self,
        owner: &str,
        name: &str,
        value: bool,
    ) -> Result<Repository> {
        let id: i32 = sqlx::query(
            r#"
            UPDATE repository
            SET default_enable_checks = $1
            WHERE owner = $2
            AND name = $3
            RETURNING id
        "#,
        )
        .bind(value)
        .bind(owner)
        .bind(name)
        .fetch_one(&mut *self.connection)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })?
        .get(0);

        self.get_from_id(id as u64).await.map(|x| x.unwrap())
    }
}

#[cfg(test)]
mod tests {
    use futures_util::FutureExt;
    use github_scbot_core::config::Config;
    use mockall::predicate;
    use sqlx::Acquire;

    use super::*;
    use crate::{errors::StdError, utils::use_temporary_db};

    type Result<T> = core::result::Result<T, StdError>;

    #[actix_rt::test]
    async fn test_automock() -> Result<()> {
        let mut mock = MockRepositoryDB::new();

        mock.expect_get()
            .with(predicate::eq("me"), predicate::eq("me"))
            .times(1)
            .returning(|_, _| {
                async {
                    Ok(Some(
                        RepositoryBuilder::default()
                            .owner("me")
                            .name("me")
                            .default_automerge(true)
                            .build()
                            .unwrap(),
                    ))
                }
                .boxed()
            });

        let repo = mock.get("me", "me").await?.unwrap();
        assert!(repo.default_automerge);
        Ok(())
    }

    #[actix_rt::test]
    async fn test_db() {
        use_temporary_db(
            Config::from_env(),
            "repository-test-db",
            |config, conn| async move {
                let mut transaction = conn.begin().await?;

                // Create
                let repo = {
                    let mut db = RepositoryDBImpl::new(&mut transaction);
                    let repo = RepositoryBuilder::default()
                        .with_config(&config)
                        .owner("me")
                        .name("me")
                        .default_strategy(GhMergeStrategy::Squash)
                        .build()?;
                    db.create(repo).await?
                };

                // Creating another repository with same name and owner should error
                {
                    let mut inner_transaction = transaction.begin().await?;
                    let mut db = RepositoryDBImpl::new(&mut inner_transaction);
                    let repo = RepositoryBuilder::default()
                        .owner("me")
                        .name("me")
                        .build()?;

                    assert!(
                        db.create(repo).await.is_err(),
                        "same repo name and owner should error"
                    );
                }

                let mut db = RepositoryDBImpl::new(&mut transaction);

                // Get
                let repo2 = db.get(repo.owner(), repo.name()).await?.unwrap();
                assert_eq!(repo2.default_strategy, GhMergeStrategy::Squash);
                assert!(!repo2.manual_interaction);

                // Get from ID
                let repo3 = db.get_from_id(repo.id()).await?.unwrap();
                assert_eq!(repo3, repo2);

                // Get unknown
                assert!(
                    db.get("unknown", "unknown").await?.is_none(),
                    "'unknown' repo should not exist"
                );

                // All
                assert_eq!(db.all().await?.len(), 1);

                // Update manual_interaction
                let repo2 = db.set_manual_interaction("me", "me", true).await?;
                assert!(repo2.manual_interaction);

                // Title validation regex
                let repo2 = db.set_pr_title_validation_regex("me", "me", "test").await?;
                assert_eq!(repo2.pr_title_validation_regex, "test");

                // Default strategy
                let repo2 = db
                    .set_default_strategy("me", "me", GhMergeStrategy::Rebase)
                    .await?;
                assert_eq!(repo2.default_strategy, GhMergeStrategy::Rebase);

                // Reviewers count
                let repo2 = db.set_default_needed_reviewers_count("me", "me", 1).await?;
                assert_eq!(repo2.default_needed_reviewers_count, 1);

                // Automerge
                let repo2 = db.set_default_automerge("me", "me", true).await?;
                assert!(repo2.default_automerge);

                // QA
                let repo2 = db.set_default_enable_qa("me", "me", true).await?;
                assert!(repo2.default_enable_qa);

                // Disable checks
                let repo2 = db.set_default_enable_checks("me", "me", false).await?;
                assert!(!repo2.default_enable_checks);

                // Delete repository
                assert!(db.delete("me", "me").await?);
                assert!(
                    db.get("me", "me").await?.is_none(),
                    "'me' repo should not exist anymore"
                );

                Ok(())
            },
        )
        .await
    }
}
