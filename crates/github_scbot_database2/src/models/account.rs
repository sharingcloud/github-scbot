use async_trait::async_trait;
use github_scbot_database_macros::SCGetter;
use sqlx::{postgres::PgRow, FromRow, PgConnection, PgPool, Postgres, Row, Transaction};

use crate::{errors::Result, DatabaseError};

#[derive(SCGetter, Debug, Clone, Default, derive_builder::Builder)]
#[builder(default)]
pub struct Account {
    #[get_deref]
    username: String,
    #[get]
    is_admin: bool,
}

impl Account {
    pub fn builder() -> AccountBuilder {
        AccountBuilder::default()
    }
}

impl<'r> FromRow<'r, PgRow> for Account {
    fn from_row(row: &'r PgRow) -> core::result::Result<Self, sqlx::Error> {
        Ok(Self {
            username: row.try_get("username")?,
            is_admin: row.try_get("is_admin")?,
        })
    }
}

#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait AccountDB {
    async fn create(&mut self, instance: Account) -> Result<Account>;
    async fn get(&mut self, username: &str) -> Result<Option<Account>>;
    async fn delete(&mut self, username: &str) -> Result<bool>;
    async fn list_admins(&mut self) -> Result<Vec<Account>>;
    async fn set_is_admin(&mut self, username: &str, value: bool) -> Result<Account>;
}

pub struct AccountDBImpl<'a> {
    connection: &'a mut PgConnection,
}

impl<'a> AccountDBImpl<'a> {
    pub fn new(connection: &'a mut PgConnection) -> Self {
        Self { connection }
    }
}

pub struct AccountDBImplPool {
    pool: PgPool,
}

impl AccountDBImplPool {
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
impl AccountDB for AccountDBImplPool {
    async fn create(&mut self, instance: Account) -> Result<Account> {
        let mut transaction = self.begin().await?;
        let data = AccountDBImpl::new(&mut *transaction)
            .create(instance)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn get(&mut self, username: &str) -> Result<Option<Account>> {
        let mut transaction = self.begin().await?;
        let data = AccountDBImpl::new(&mut *transaction).get(username).await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn delete(&mut self, username: &str) -> Result<bool> {
        let mut transaction = self.begin().await?;
        let data = AccountDBImpl::new(&mut *transaction)
            .delete(username)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn list_admins(&mut self) -> Result<Vec<Account>> {
        let mut transaction = self.begin().await?;
        let data = AccountDBImpl::new(&mut *transaction).list_admins().await?;
        self.commit(transaction).await?;
        Ok(data)
    }

    async fn set_is_admin(&mut self, username: &str, value: bool) -> Result<Account> {
        let mut transaction = self.begin().await?;
        let data = AccountDBImpl::new(&mut *transaction)
            .set_is_admin(username, value)
            .await?;
        self.commit(transaction).await?;
        Ok(data)
    }
}

#[async_trait]
impl<'a> AccountDB for AccountDBImpl<'a> {
    async fn create(&mut self, instance: Account) -> Result<Account> {
        let username: String = sqlx::query(
            r#"
            INSERT INTO account
            (
                username,
                is_admin
            )
            VALUES
            (
                $1,
                $2
            )
            RETURNING username
            ;
        "#,
        )
        .bind(instance.username)
        .bind(instance.is_admin)
        .fetch_one(&mut *self.connection)
        .await
        .map_err(DatabaseError::SqlError)?
        .get(0);

        self.get(&username).await.map(|x| x.unwrap())
    }

    async fn get(&mut self, username: &str) -> Result<Option<Account>> {
        sqlx::query_as::<_, Account>(
            r#"
                SELECT *
                FROM account
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
            DELETE FROM account
            WHERE username = $1
        "#,
        )
        .bind(username)
        .execute(&mut *self.connection)
        .await
        .map(|x| x.rows_affected() > 0)
        .map_err(DatabaseError::SqlError)
    }

    async fn list_admins(&mut self) -> Result<Vec<Account>> {
        sqlx::query_as::<_, Account>(
            r#"
                SELECT *
                FROM account
                WHERE is_admin = $1
            "#,
        )
        .bind(true)
        .fetch_all(&mut *self.connection)
        .await
        .map_err(DatabaseError::SqlError)
    }

    async fn set_is_admin(&mut self, username: &str, value: bool) -> Result<Account> {
        let username: String = sqlx::query(
            r#"
            UPDATE repository
            SET is_admin = $1
            WHERE username = $2
            RETURNING username
        "#,
        )
        .bind(value)
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

    use crate::{
        models::{
            account::{Account, AccountDB, AccountDBImplPool},
            repository::{Repository, RepositoryDB, RepositoryDBImplPool},
        },
        utils::use_temporary_db,
    };

    #[actix_rt::test]
    async fn test_db() {
        use_temporary_db(
            Config::from_env(),
            "account-test-db",
            |_config, conn| async move {
                let mut repo_db = RepositoryDBImplPool::new(conn.clone());
                let mut account_db = AccountDBImplPool::new(conn.clone());

                async fn sample(repo_db: &mut dyn RepositoryDB, account_db: &mut dyn AccountDB) {
                    let _repo = repo_db
                        .create(
                            Repository::builder()
                                .owner("me".into())
                                .name("me".into())
                                .build()
                                .unwrap(),
                        )
                        .await
                        .unwrap();
                    let _account = account_db
                        .create(Account::builder().username("me".into()).build().unwrap())
                        .await
                        .unwrap();

                    assert!(repo_db.get("me", "me").await.unwrap().is_some());
                    assert!(account_db.get("me").await.unwrap().is_some());
                }

                sample(&mut repo_db, &mut account_db).await;
                Ok(())
            },
        )
        .await;
    }
}
