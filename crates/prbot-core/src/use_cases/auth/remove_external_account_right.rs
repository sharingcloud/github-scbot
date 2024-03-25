use prbot_models::RepositoryPath;

use crate::{CoreContext, Result};

pub struct RemoveExternalAccountRight;

impl RemoveExternalAccountRight {
    #[tracing::instrument(skip(self, ctx), fields(username, repository_path))]
    pub async fn run(
        &self,
        ctx: &CoreContext<'_>,
        repository_path: &RepositoryPath,
        username: &str,
    ) -> Result<()> {
        let (owner, name) = repository_path.components();

        ctx.db_service
            .external_account_rights_delete(owner, name, username)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use prbot_database_interface::DbService;
    use prbot_models::{ExternalAccount, ExternalAccountRight, Repository};

    use super::RemoveExternalAccountRight;
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

        RemoveExternalAccountRight
            .run(&ctx.as_context(), &("owner", "name").into(), "acc")
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
