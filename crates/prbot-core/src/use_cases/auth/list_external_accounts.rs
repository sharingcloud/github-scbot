use prbot_models::ExternalAccount;

use crate::{CoreContext, Result};

pub struct ListExternalAccounts;

impl<'a> ListExternalAccounts {
    #[tracing::instrument(skip(self, ctx))]
    pub async fn run(&self, ctx: &CoreContext<'_>) -> Result<Vec<ExternalAccount>> {
        ctx.db_service
            .external_accounts_all()
            .await
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use prbot_database_interface::DbService;
    use prbot_models::ExternalAccount;

    use super::ListExternalAccounts;
    use crate::context::tests::CoreContextTest;

    #[tokio::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let ctx = CoreContextTest::new();

        let acc = ctx
            .db_service
            .external_accounts_create(ExternalAccount {
                username: "acc".into(),
                ..Default::default()
            })
            .await?;

        assert_eq!(
            ListExternalAccounts.run(&ctx.as_context()).await?,
            vec![acc]
        );

        Ok(())
    }
}
