use github_scbot_crypto::RsaUtils;

use super::{ExternalAccountModel, IExternalAccountDbAdapter};
use crate::Result;

#[must_use]
pub struct ExternalAccountModelBuilder {
    username: String,
    public_key: Option<String>,
    private_key: Option<String>,
    is_superuser: Option<bool>,
}

impl ExternalAccountModelBuilder {
    pub fn default<T: Into<String>>(username: T) -> Self {
        Self {
            username: username.into(),
            public_key: None,
            private_key: None,
            is_superuser: None,
        }
    }

    pub fn from_model(model: &ExternalAccountModel) -> Self {
        Self {
            username: model.username.clone(),
            private_key: Some(model.private_key.clone()),
            public_key: Some(model.public_key.clone()),
            is_superuser: Some(model.is_superuser),
        }
    }

    pub fn private_key<T: Into<String>>(mut self, key: T) -> Self {
        self.private_key = Some(key.into());
        self
    }

    pub fn public_key<T: Into<String>>(mut self, key: T) -> Self {
        self.public_key = Some(key.into());
        self
    }

    pub fn is_superuser(mut self, value: bool) -> Self {
        self.is_superuser = Some(value);
        self
    }

    pub fn generate_keys(mut self) -> Self {
        let (private_key, public_key) = RsaUtils::generate_rsa_keys();
        self.private_key = Some(private_key.to_string());
        self.public_key = Some(public_key.to_string());
        self
    }

    pub async fn create_or_update(
        self,
        db_adapter: &dyn IExternalAccountDbAdapter,
    ) -> Result<ExternalAccountModel> {
        let mut handle = match db_adapter.get_from_username(&self.username).await {
            Ok(entry) => entry,
            Err(_) => db_adapter.create(self.build()).await?,
        };

        handle.public_key = match self.public_key {
            Some(k) => k,
            None => handle.public_key,
        };
        handle.private_key = match self.private_key {
            Some(k) => k,
            None => handle.private_key,
        };
        handle.is_superuser = match self.is_superuser {
            Some(v) => v,
            None => handle.is_superuser,
        };
        db_adapter.save(&mut handle).await?;

        Ok(handle)
    }

    pub fn build(&self) -> ExternalAccountModel {
        ExternalAccountModel {
            username: self.username.clone(),
            public_key: self.public_key.clone().unwrap_or_else(String::new),
            private_key: self.private_key.clone().unwrap_or_else(String::new),
            is_superuser: self.is_superuser.unwrap_or(false)
        }
    }
}
