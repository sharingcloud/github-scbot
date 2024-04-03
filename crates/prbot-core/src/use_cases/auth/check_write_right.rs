use prbot_ghapi_interface::types::GhUserPermission;

use super::check_is_admin::CheckIsAdmin;
use crate::{CoreContext, Result};

pub struct CheckWriteRight;

impl CheckWriteRight {
    #[tracing::instrument(skip(self, ctx), fields(username, user_permission), ret)]
    pub async fn run(
        &self,
        ctx: &CoreContext<'_>,
        username: &str,
        user_permission: GhUserPermission,
    ) -> Result<bool> {
        let is_admin = CheckIsAdmin.run(ctx, username).await?;

        Ok(is_admin || user_permission.can_write())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use prbot_database_interface::DbService;
    use prbot_ghapi_interface::types::GhUserPermission;
    use prbot_models::Account;

    use super::CheckWriteRight;
    use crate::context::tests::CoreContextTest;

    #[tokio::test]
    async fn run_read_not_admin() -> Result<(), Box<dyn Error>> {
        let ctx = CoreContextTest::new();

        ctx.db_service
            .accounts_create(Account {
                username: "me".into(),
                is_admin: false,
            })
            .await?;

        assert!(
            !CheckWriteRight
                .run(&ctx.as_context(), "me", GhUserPermission::Read)
                .await?
        );

        Ok(())
    }

    #[tokio::test]
    async fn run_read_admin() -> Result<(), Box<dyn Error>> {
        let ctx = CoreContextTest::new();

        ctx.db_service
            .accounts_create(Account {
                username: "me".into(),
                is_admin: true,
            })
            .await?;

        assert!(
            CheckWriteRight
                .run(&ctx.as_context(), "me", GhUserPermission::Read)
                .await?
        );

        Ok(())
    }

    #[tokio::test]
    async fn run_write_admin() -> Result<(), Box<dyn Error>> {
        let ctx = CoreContextTest::new();

        ctx.db_service
            .accounts_create(Account {
                username: "me".into(),
                is_admin: true,
            })
            .await?;

        assert!(
            CheckWriteRight
                .run(&ctx.as_context(), "me", GhUserPermission::Write)
                .await?
        );

        Ok(())
    }

    #[tokio::test]
    async fn run_write_not_admin() -> Result<(), Box<dyn Error>> {
        let ctx = CoreContextTest::new();

        ctx.db_service
            .accounts_create(Account {
                username: "me".into(),
                is_admin: false,
            })
            .await?;

        assert!(
            CheckWriteRight
                .run(&ctx.as_context(), "me", GhUserPermission::Write)
                .await?
        );

        Ok(())
    }
}
