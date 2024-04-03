use crate::{CoreContext, Result};

pub struct RemoveExternalAccount;

impl RemoveExternalAccount {
    #[tracing::instrument(skip(self, ctx), fields(username))]
    pub async fn run(&self, ctx: &CoreContext<'_>, username: &str) -> Result<()> {
        ctx.db_service.external_accounts_delete(username).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use prbot_database_interface::DbService;
    use prbot_models::ExternalAccount;

    use super::RemoveExternalAccount;
    use crate::context::tests::CoreContextTest;

    #[tokio::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let ctx = CoreContextTest::new();

        ctx.db_service
            .external_accounts_create(ExternalAccount {
                username: "acc".into(),
                ..Default::default()
            })
            .await?;

        RemoveExternalAccount.run(&ctx.as_context(), "acc").await?;

        assert_eq!(ctx.db_service.external_accounts_get("acc").await?, None);

        Ok(())
    }
}
