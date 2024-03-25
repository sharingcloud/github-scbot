use crate::{CoreContext, Result};

pub struct RemoveAllExternalAccountRights;

impl RemoveAllExternalAccountRights {
    #[tracing::instrument(skip(self, ctx), fields(username))]
    pub async fn run(&self, ctx: &CoreContext<'_>, username: &str) -> Result<()> {
        ctx.db_service
            .external_account_rights_delete_all(username)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use prbot_database_interface::DbService;
    use prbot_models::{ExternalAccount, ExternalAccountRight, Repository};

    use super::RemoveAllExternalAccountRights;
    use crate::context::tests::CoreContextTest;

    #[tokio::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let ctx = CoreContextTest::new();

        let repo = ctx
            .db_service
            .repositories_create(Repository {
                owner: "owner".into(),
                name: "name".into(),
                ..Default::default()
            })
            .await?;

        ctx.db_service
            .external_accounts_create(ExternalAccount {
                username: "acc".into(),
                ..Default::default()
            })
            .await?;

        ctx.db_service
            .external_account_rights_create(ExternalAccountRight {
                repository_id: repo.id,
                username: "acc".into(),
            })
            .await?;

        RemoveAllExternalAccountRights
            .run(&ctx.as_context(), "acc")
            .await?;

        assert_eq!(
            ctx.db_service
                .external_account_rights_get("owner", "name", "acc")
                .await?,
            None
        );

        Ok(())
    }
}
