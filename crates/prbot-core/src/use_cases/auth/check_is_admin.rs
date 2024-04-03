use crate::{CoreContext, Result};

pub struct CheckIsAdmin;

impl CheckIsAdmin {
    #[tracing::instrument(skip(self, ctx), fields(username), ret)]
    pub async fn run(&self, ctx: &CoreContext<'_>, username: &str) -> Result<bool> {
        let known_admins: Vec<_> = ctx.db_service.accounts_list_admins().await?;

        Ok(known_admins.iter().any(|acc| acc.username == username))
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use prbot_database_interface::DbService;
    use prbot_models::Account;

    use super::CheckIsAdmin;
    use crate::context::tests::CoreContextTest;

    #[tokio::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let ctx = CoreContextTest::new();

        ctx.db_service
            .accounts_create(Account {
                username: "me".into(),
                is_admin: true,
            })
            .await?;

        assert!(CheckIsAdmin.run(&ctx.as_context(), "me").await?);

        Ok(())
    }

    #[tokio::test]
    async fn run_not_admin() -> Result<(), Box<dyn Error>> {
        let ctx = CoreContextTest::new();

        ctx.db_service
            .accounts_create(Account {
                username: "me".into(),
                is_admin: false,
            })
            .await?;

        assert!(!CheckIsAdmin.run(&ctx.as_context(), "me").await?);

        Ok(())
    }
}
