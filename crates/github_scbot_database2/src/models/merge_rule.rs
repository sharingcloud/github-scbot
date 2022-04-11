use async_trait::async_trait;
use github_scbot_database_macros::SCGetter;
use github_scbot_types::pulls::GhMergeStrategy;
use sqlx::{postgres::PgRow, FromRow, PgConnection, PgPool, Postgres, Row, Transaction};

use crate::{
    errors::Result,
    fields::{GhMergeStrategyDecode, RuleBranch, RuleBranchDecode},
    DatabaseError,
};

#[derive(SCGetter, Debug, Clone, Default, derive_builder::Builder)]
#[builder(default)]
pub struct MergeRule {
    #[get]
    repository_id: u64,
    #[get_ref]
    base_branch: RuleBranch,
    #[get_ref]
    head_branch: RuleBranch,
    #[get]
    strategy: GhMergeStrategy,
}

impl MergeRule {
    pub fn builder() -> MergeRuleBuilder {
        MergeRuleBuilder::default()
    }
}

impl<'r> FromRow<'r, PgRow> for MergeRule {
    fn from_row(row: &'r PgRow) -> core::result::Result<Self, sqlx::Error> {
        Ok(Self {
            repository_id: row.try_get::<i32, _>("repository_id")? as u64,
            base_branch: row.try_get::<RuleBranchDecode, _>("base_branch")?.clone(),
            head_branch: row.try_get::<RuleBranchDecode, _>("head_branch")?.clone(),
            strategy: *row.try_get::<GhMergeStrategyDecode, _>("strategy")?,
        })
    }
}

#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait MergeRuleDB {
    async fn create(&mut self, instance: MergeRule) -> Result<MergeRule>;
    async fn get(
        &mut self,
        owner: &str,
        name: &str,
        base_branch: RuleBranch,
        head_branch: RuleBranch,
    ) -> Result<Option<MergeRule>>;
    async fn delete(
        &mut self,
        owner: &str,
        name: &str,
        base_branch: RuleBranch,
        head_branch: RuleBranch,
    ) -> Result<bool>;
    async fn list(&mut self, owner: &str, name: &str) -> Result<Vec<MergeRule>>;
}

pub struct MergeRuleDBImpl<'a> {
    connection: &'a mut PgConnection,
}

impl<'a> MergeRuleDBImpl<'a> {
    pub fn new(connection: &'a mut PgConnection) -> Self {
        Self { connection }
    }

    async fn get_from_id(&mut self, id: i32) -> Result<Option<MergeRule>> {
        sqlx::query_as::<_, MergeRule>(
            r#"
            SELECT *
            FROM merge_rule
            WHERE id = $1
        "#,
        )
        .bind(id)
        .fetch_optional(&mut *self.connection)
        .await
        .map_err(DatabaseError::SqlError)
    }
}

pub struct MergeRuleDBImplPool {
    pool: PgPool,
}

impl MergeRuleDBImplPool {
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
impl MergeRuleDB for MergeRuleDBImplPool {
    async fn create(&mut self, instance: MergeRule) -> Result<MergeRule> {
        let mut transaction = self.begin().await?;
        let data = MergeRuleDBImpl::new(&mut *transaction)
            .create(instance)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn get(
        &mut self,
        owner: &str,
        name: &str,
        base_branch: RuleBranch,
        head_branch: RuleBranch,
    ) -> Result<Option<MergeRule>> {
        let mut transaction = self.begin().await?;
        let data = MergeRuleDBImpl::new(&mut *transaction)
            .get(owner, name, base_branch, head_branch)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn delete(
        &mut self,
        owner: &str,
        name: &str,
        base_branch: RuleBranch,
        head_branch: RuleBranch,
    ) -> Result<bool> {
        let mut transaction = self.begin().await?;
        let data = MergeRuleDBImpl::new(&mut *transaction)
            .delete(owner, name, base_branch, head_branch)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn list(&mut self, owner: &str, name: &str) -> Result<Vec<MergeRule>> {
        let mut transaction = self.begin().await?;
        let data = MergeRuleDBImpl::new(&mut *transaction)
            .list(owner, name)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }
}

#[async_trait]
impl<'a> MergeRuleDB for MergeRuleDBImpl<'a> {
    async fn create(&mut self, instance: MergeRule) -> Result<MergeRule> {
        let new_id: i32 = sqlx::query(
            r#"
            INSERT INTO merge_rule
            (
                repository_id,
                base_branch,
                head_branch,
                strategy
            )
            VALUES
            (
                $1,
                $2,
                $3,
                $4
            )
            RETURNING id
            ;
        "#,
        )
        .bind(instance.repository_id as i32)
        .bind(instance.base_branch.to_string())
        .bind(instance.head_branch.to_string())
        .bind(instance.strategy.to_string())
        .fetch_one(&mut *self.connection)
        .await
        .map_err(DatabaseError::SqlError)?
        .get(0);

        self.get_from_id(new_id).await.map(|x| x.unwrap())
    }

    async fn get(
        &mut self,
        owner: &str,
        name: &str,
        base_branch: RuleBranch,
        head_branch: RuleBranch,
    ) -> Result<Option<MergeRule>> {
        sqlx::query_as::<_, MergeRule>(
            r#"
            SELECT *
            FROM merge_rule
            INNER JOIN repository ON (repository.owner = $1 AND repository.name = $2)
            WHERE base_branch = $3
            AND head_branch = $4;
        "#,
        )
        .bind(owner)
        .bind(name)
        .bind(base_branch.to_string())
        .bind(head_branch.to_string())
        .fetch_optional(&mut *self.connection)
        .await
        .map_err(DatabaseError::SqlError)
    }

    async fn delete(
        &mut self,
        owner: &str,
        name: &str,
        base_branch: RuleBranch,
        head_branch: RuleBranch,
    ) -> Result<bool> {
        sqlx::query(
            r#"
            DELETE
            FROM merge_rule
            USING repository
            WHERE repository.owner = $1
            AND repository.name = $2
            AND base_branch = $3
            AND head_branch = $4;
        "#,
        )
        .bind(owner)
        .bind(name)
        .bind(base_branch.to_string())
        .bind(head_branch.to_string())
        .execute(&mut *self.connection)
        .await
        .map(|x| x.rows_affected() > 0)
        .map_err(DatabaseError::SqlError)
    }

    async fn list(&mut self, owner: &str, name: &str) -> Result<Vec<MergeRule>> {
        sqlx::query_as::<_, MergeRule>(
            r#"
            SELECT *
            FROM merge_rule
            INNER JOIN repository ON (repository.owner = $1 AND repository.name = $2)
        "#,
        )
        .bind(owner)
        .bind(name)
        .fetch_all(&mut *self.connection)
        .await
        .map_err(DatabaseError::SqlError)
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_conf::Config;
    use github_scbot_types::pulls::GhMergeStrategy;

    use crate::{
        fields::RuleBranch,
        models::{
            merge_rule::{MergeRule, MergeRuleDB, MergeRuleDBImpl},
            repository::{Repository, RepositoryDB},
        },
        utils::use_temporary_db,
        RepositoryDBImpl,
    };

    #[actix_rt::test]
    async fn test_db() {
        use_temporary_db(
            Config::from_env(),
            "merge-rule-test-db",
            |_config, conn| async move {
                let mut conn = conn.acquire().await?;
                let repo = {
                    let mut db = RepositoryDBImpl::new(&mut conn);
                    db.create(Repository::builder().build()?).await?
                };

                let mut db = MergeRuleDBImpl::new(&mut conn);
                let _rule = db
                    .create(
                        MergeRule::builder()
                            .repository_id(repo.id())
                            .strategy(GhMergeStrategy::Squash)
                            .build()?,
                    )
                    .await?;
                assert!(db
                    .get("", "", RuleBranch::Wildcard, RuleBranch::Wildcard)
                    .await?
                    .is_some());
                assert!(db
                    .get(
                        "",
                        "",
                        RuleBranch::Named("nope".into()),
                        RuleBranch::Wildcard
                    )
                    .await?
                    .is_none());

                assert!(
                    db.delete("", "", RuleBranch::Wildcard, RuleBranch::Wildcard)
                        .await?
                );

                Ok(())
            },
        )
        .await;
    }
}
