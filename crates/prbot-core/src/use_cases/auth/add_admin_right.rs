use prbot_models::Account;

use crate::{CoreContext, Result};

pub struct AddAdminRight;

impl AddAdminRight {
    #[tracing::instrument(skip(self, ctx), fields(username))]
    pub async fn run(&self, ctx: &CoreContext<'_>, username: &str) -> Result<()> {
        match ctx.db_service.accounts_get(username).await? {
            Some(_) => ctx.db_service.accounts_set_is_admin(username, true).await?,
            None => {
                ctx.db_service
                    .accounts_create(Account {
                        username: username.into(),
                        is_admin: true,
                    })
                    .await?
            }
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use prbot_database_interface::DbService;

    use super::AddAdminRight;
    use crate::context::tests::CoreContextTest;

    #[tokio::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let ctx = CoreContextTest::new();

        AddAdminRight.run(&ctx.as_context(), "me").await?;
        assert!(ctx.db_service.accounts_get_expect("me").await?.is_admin);

        Ok(())
    }
}
