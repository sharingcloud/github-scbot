use prbot_models::Account;

use crate::{CoreContext, Result};

pub struct RemoveAdminRight;

impl RemoveAdminRight {
    #[tracing::instrument(skip(self, ctx), fields(username))]
    pub async fn run(&self, ctx: &CoreContext<'_>, username: &str) -> Result<()> {
        match ctx.db_service.accounts_get(username).await? {
            Some(_) => {
                ctx.db_service
                    .accounts_set_is_admin(username, false)
                    .await?
            }
            None => {
                ctx.db_service
                    .accounts_create(Account {
                        username: username.to_string(),
                        is_admin: false,
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
    use prbot_models::Account;

    use super::RemoveAdminRight;
    use crate::context::tests::CoreContextTest;

    #[tokio::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let ctx = CoreContextTest::new();

        ctx.db_service
            .accounts_create(Account {
                username: "acc".into(),
                is_admin: true,
            })
            .await?;

        RemoveAdminRight.run(&ctx.as_context(), "acc").await?;

        assert!(!ctx.db_service.accounts_get_expect("acc").await?.is_admin);

        Ok(())
    }
}
