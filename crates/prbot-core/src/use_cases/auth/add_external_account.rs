use prbot_models::ExternalAccount;

use crate::{CoreContext, Result};

pub struct AddExternalAccount;

impl AddExternalAccount {
    #[tracing::instrument(skip(self, ctx), fields(username))]
    pub async fn run(&self, ctx: &CoreContext<'_>, username: &str) -> Result<()> {
        ctx.db_service
            .external_accounts_create(
                ExternalAccount {
                    username: username.into(),
                    ..Default::default()
                }
                .with_generated_keys(),
            )
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use prbot_database_interface::DbService;

    use super::AddExternalAccount;
    use crate::context::tests::CoreContextTest;

    #[tokio::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let ctx = CoreContextTest::new();

        AddExternalAccount.run(&ctx.as_context(), "me").await?;

        assert!(ctx.db_service.external_accounts_get("me").await?.is_some());

        Ok(())
    }
}
