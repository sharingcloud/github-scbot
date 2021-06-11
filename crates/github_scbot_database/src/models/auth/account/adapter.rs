use async_trait::async_trait;
use diesel::prelude::*;
use tokio_diesel::AsyncRunQueryDsl;

use super::AccountModel;
use crate::{schema::account, DatabaseError, DbPool, Result};

/// Account DB adapter.
#[async_trait]
pub trait IAccountDbAdapter {
    /// Creates a new account.
    async fn create(&self, entry: AccountModel) -> Result<AccountModel>;
    /// Gets account from username.
    async fn get_from_username(&self, username: &str) -> Result<AccountModel>;
    /// Lists available accounts.
    async fn list(&self) -> Result<Vec<AccountModel>>;
    /// Lists available admin accounts.
    async fn list_admin_accounts(&self) -> Result<Vec<AccountModel>>;
    /// Removes a specific account.
    async fn remove(&self, entry: AccountModel) -> Result<()>;
    /// Saves and updates a specific account.
    async fn save(&self, entry: &mut AccountModel) -> Result<()>;
}

/// Concrete account DB adapter.
pub struct AccountDbAdapter<'a> {
    pool: &'a DbPool,
}

impl<'a> AccountDbAdapter<'a> {
    /// Creates a new account DB adapter.
    pub fn new(pool: &'a DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<'a> IAccountDbAdapter for AccountDbAdapter<'a> {
    async fn create(&self, entry: AccountModel) -> Result<AccountModel> {
        diesel::insert_into(account::table)
            .values(entry)
            .get_result_async(&self.pool)
            .await
            .map_err(DatabaseError::from)
    }

    async fn get_from_username(&self, username: &str) -> Result<AccountModel> {
        let username = username.to_owned();

        account::table
            .filter(account::username.eq(username.clone()))
            .first_async(&self.pool)
            .await
            .map_err(|_e| DatabaseError::UnknownAccount(username.to_string()))
    }

    async fn list(&self) -> Result<Vec<AccountModel>> {
        account::table
            .load_async::<AccountModel>(&self.pool)
            .await
            .map_err(DatabaseError::from)
    }

    async fn list_admin_accounts(&self) -> Result<Vec<AccountModel>> {
        account::table
            .filter(account::is_admin.eq(true))
            .load_async::<AccountModel>(&self.pool)
            .await
            .map_err(DatabaseError::from)
    }

    async fn remove(&self, entry: AccountModel) -> Result<()> {
        diesel::delete(account::table.filter(account::username.eq(entry.username.clone())))
            .execute_async(&self.pool)
            .await?;

        Ok(())
    }

    async fn save(&self, entry: &mut AccountModel) -> Result<()> {
        let copy = entry.clone();

        *entry = diesel::update(account::table.filter(account::username.eq(copy.username.clone())))
            .set(copy)
            .get_result_async(&self.pool)
            .await
            .map_err(DatabaseError::from)?;

        Ok(())
    }
}

/// Dummy account DB adapter.
#[derive(Clone)]
pub struct DummyAccountDbAdapter {
    /// Create response.
    pub create_response: Option<Result<AccountModel>>,
    /// Get from username response.
    pub get_from_username_response: Result<AccountModel>,
    /// List response.
    pub list_response: Result<Vec<AccountModel>>,
    /// List admin accounts response.
    pub list_admin_accounts_response: Result<Vec<AccountModel>>,
    /// Remove response.
    pub remove_response: Result<()>,
    /// Save response.
    pub save_response: Result<()>,
}

impl Default for DummyAccountDbAdapter {
    fn default() -> Self {
        Self {
            create_response: None,
            get_from_username_response: Err(DatabaseError::UnknownAccount("test".into())),
            list_response: Ok(Vec::new()),
            list_admin_accounts_response: Ok(Vec::new()),
            remove_response: Ok(()),
            save_response: Ok(()),
        }
    }
}

impl DummyAccountDbAdapter {
    /// Creates a new dummy account DB adapter.
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
#[allow(unused_variables)]
impl IAccountDbAdapter for DummyAccountDbAdapter {
    async fn create(&self, entry: AccountModel) -> Result<AccountModel> {
        self.create_response
            .as_ref()
            .map_or(Ok(entry), Clone::clone)
    }

    async fn get_from_username(&self, username: &str) -> Result<AccountModel> {
        self.get_from_username_response.clone()
    }

    async fn list(&self) -> Result<Vec<AccountModel>> {
        self.list_response.clone()
    }

    async fn list_admin_accounts(&self) -> Result<Vec<AccountModel>> {
        self.list_admin_accounts_response.clone()
    }

    async fn remove(&self, entry: AccountModel) -> Result<()> {
        self.remove_response.clone()
    }

    async fn save(&self, entry: &mut AccountModel) -> Result<()> {
        self.save_response.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::using_test_db;

    #[actix_rt::test]
    async fn test_create() -> Result<()> {
        using_test_db("auth_adapter_test_create", |_config, pool| async move {
            let db_adapter = AccountDbAdapter::new(&pool);
            let account_model = AccountModel::builder("test").build();
            let account_model = db_adapter.create(account_model).await?;
            assert_eq!(
                account_model,
                AccountModel {
                    username: "test".into(),
                    is_admin: false
                }
            );

            Ok::<_, DatabaseError>(())
        })
        .await
    }

    #[actix_rt::test]
    async fn test_get_from_username() -> Result<()> {
        using_test_db(
            "auth_adapter_test_get_from_username",
            |_config, pool| async move {
                let db_adapter = AccountDbAdapter::new(&pool);
                let account_model = AccountModel::builder("test")
                    .create_or_update(&db_adapter)
                    .await?;
                assert_eq!(db_adapter.get_from_username("test").await?, account_model);

                Ok::<_, DatabaseError>(())
            },
        )
        .await
    }

    #[actix_rt::test]
    async fn test_list() -> Result<()> {
        using_test_db("auth_adapter_list", |_config, pool| async move {
            let db_adapter = AccountDbAdapter::new(&pool);
            assert_eq!(db_adapter.list().await?, Vec::new());

            let test = AccountModel::builder("test")
                .create_or_update(&db_adapter)
                .await?;
            let test2 = AccountModel::builder("test2")
                .create_or_update(&db_adapter)
                .await?;
            assert_eq!(db_adapter.list().await?, vec![test, test2]);

            Ok::<_, DatabaseError>(())
        })
        .await
    }

    #[actix_rt::test]
    async fn test_list_admin_accounts() -> Result<()> {
        using_test_db(
            "auth_adapter_list_admin_accounts",
            |_config, pool| async move {
                let db_adapter = AccountDbAdapter::new(&pool);

                AccountModel::builder("not_admin")
                    .create_or_update(&db_adapter)
                    .await?;
                let admin = AccountModel::builder("admin")
                    .admin(true)
                    .create_or_update(&db_adapter)
                    .await?;
                assert_eq!(db_adapter.list_admin_accounts().await?, vec![admin]);

                Ok::<_, DatabaseError>(())
            },
        )
        .await
    }

    #[actix_rt::test]
    async fn test_remove() -> Result<()> {
        using_test_db("auth_adapter_remove", |_config, pool| async move {
            let db_adapter = AccountDbAdapter::new(&pool);

            let account = AccountModel::builder("test")
                .create_or_update(&db_adapter)
                .await?;
            assert_eq!(db_adapter.get_from_username("test").await?, account);

            db_adapter.remove(account).await?;
            assert!(matches!(
                db_adapter.get_from_username("test").await,
                Err(DatabaseError::UnknownAccount(_))
            ));

            Ok::<_, DatabaseError>(())
        })
        .await
    }

    #[actix_rt::test]
    async fn test_save() -> Result<()> {
        using_test_db("auth_adapter_save", |_config, pool| async move {
            let db_adapter = AccountDbAdapter::new(&pool);

            let mut account = AccountModel::builder("test")
                .create_or_update(&db_adapter)
                .await?;
            account.is_admin = true;

            // Admin change is not saved to database.
            assert_eq!(
                db_adapter.get_from_username("test").await?,
                AccountModel {
                    is_admin: false,
                    ..account.clone()
                }
            );

            db_adapter.save(&mut account).await?;
            assert_eq!(db_adapter.get_from_username("test").await?, account);

            Ok::<_, DatabaseError>(())
        })
        .await
    }
}