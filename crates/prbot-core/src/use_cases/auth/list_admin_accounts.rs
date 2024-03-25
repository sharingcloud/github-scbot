use prbot_models::Account;

use crate::{CoreContext, Result};

pub struct ListAdminAccounts;

impl ListAdminAccounts {
    #[tracing::instrument(skip(self, ctx))]
    pub async fn run(&self, ctx: &CoreContext<'_>) -> Result<Vec<Account>> {
        ctx.db_service
            .accounts_list_admins()
            .await
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use prbot_database_interface::DbService;
    use prbot_models::Account;

    use super::ListAdminAccounts;
    use crate::context::tests::CoreContextTest;

    #[tokio::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let ctx = CoreContextTest::new();

        let one = ctx
            .db_service
            .accounts_create(Account {
                username: "one".into(),
                is_admin: true,
            })
            .await?;

        let two = ctx
            .db_service
            .accounts_create(Account {
                username: "two".into(),
                is_admin: true,
            })
            .await?;

        assert_eq!(
            ListAdminAccounts.run(&ctx.as_context()).await?,
            vec![one, two]
        );

        Ok(())
    }
}
