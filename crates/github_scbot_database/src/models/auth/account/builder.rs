use super::{AccountModel, IAccountDbAdapter};
use crate::Result;

#[must_use]
pub struct AccountModelBuilder {
    username: String,
    is_admin: Option<bool>,
}

impl AccountModelBuilder {
    pub fn default<T: Into<String>>(username: T) -> Self {
        Self {
            username: username.into(),
            is_admin: None,
        }
    }

    pub fn from_model(model: &AccountModel) -> Self {
        Self {
            username: model.username.clone(),
            is_admin: Some(model.is_admin),
        }
    }

    pub fn admin(mut self, value: bool) -> Self {
        self.is_admin = Some(value);
        self
    }

    pub async fn create_or_update(
        self,
        db_adapter: &dyn IAccountDbAdapter,
    ) -> Result<AccountModel> {
        let mut handle = match db_adapter.get_from_username(&self.username).await {
            Ok(entry) => entry,
            Err(_) => db_adapter.create(self.build()).await?,
        };

        handle.is_admin = match self.is_admin {
            Some(a) => a,
            None => handle.is_admin,
        };
        db_adapter.save(&mut handle).await?;
        Ok(handle)
    }

    pub fn build(&self) -> AccountModel {
        AccountModel {
            username: self.username.clone(),
            is_admin: self.is_admin.unwrap_or(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::DummyAccountDbAdapter;

    #[test]
    fn test_build() {
        assert_eq!(
            AccountModelBuilder::default("username").admin(true).build(),
            AccountModel {
                is_admin: true,
                username: "username".into()
            }
        );

        assert_eq!(
            AccountModelBuilder::default("test").build(),
            AccountModel {
                is_admin: false,
                username: "test".into()
            }
        );

        let account = AccountModelBuilder::default("test").build();
        let another_account = AccountModelBuilder::from_model(&account)
            .admin(true)
            .build();
        assert_eq!(
            another_account,
            AccountModel {
                is_admin: true,
                username: "test".into()
            }
        );
    }

    #[actix_rt::test]
    async fn test_create_or_update() -> Result<()> {
        let mut db_adapter = DummyAccountDbAdapter::new();

        let new_account = AccountModelBuilder::default("new")
            .admin(true)
            .create_or_update(&db_adapter)
            .await?;
        assert_eq!(
            new_account,
            AccountModel {
                is_admin: true,
                username: "new".into()
            }
        );

        db_adapter
            .get_from_username_response
            .set_callback(Box::new(move |_| Ok(new_account.clone())));

        let updated_account = AccountModelBuilder::default("new")
            .create_or_update(&db_adapter)
            .await?;
        assert_eq!(
            updated_account,
            AccountModel {
                is_admin: true,
                username: "new".into()
            }
        );

        Ok(())
    }
}
