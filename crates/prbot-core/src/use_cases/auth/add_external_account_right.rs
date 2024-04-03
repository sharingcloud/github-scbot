use prbot_models::{ExternalAccountRight, RepositoryPath};

use crate::{CoreContext, Result};

pub struct AddExternalAccountRight;

impl AddExternalAccountRight {
    #[tracing::instrument(skip(self, ctx), fields(repository_path, username))]
    pub async fn run(
        &self,
        ctx: &CoreContext<'_>,
        repository_path: &RepositoryPath,
        username: &str,
    ) -> Result<()> {
        let (owner, name) = repository_path.components();
        let repository = ctx.db_service.repositories_get_expect(owner, name).await?;

        ctx.db_service
            .external_account_rights_delete(owner, name, username)
            .await?;
        ctx.db_service
            .external_account_rights_create(ExternalAccountRight {
                repository_id: repository.id,
                username: username.into(),
            })
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use prbot_database_interface::DbService;
    use prbot_models::{ExternalAccount, Repository};

    use super::AddExternalAccountRight;
    use crate::context::tests::CoreContextTest;

    #[tokio::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let ctx = CoreContextTest::new();

        let repository = ctx
            .db_service
            .repositories_create(Repository {
                owner: "owner".into(),
                name: "name".into(),
                ..Default::default()
            })
            .await?;
        ctx.db_service
            .external_accounts_create(ExternalAccount {
                username: "me".into(),
                ..Default::default()
            })
            .await?;

        AddExternalAccountRight
            .run(&ctx.as_context(), &repository.path(), "me")
            .await?;

        assert!(ctx
            .db_service
            .external_account_rights_get("owner", "name", "me")
            .await?
            .is_some());

        Ok(())
    }
}
