use prbot_models::Repository;

use crate::{CoreContext, Result};

pub struct ListExternalAccountRights;

impl<'a> ListExternalAccountRights {
    #[tracing::instrument(skip(self, ctx), fields(username))]
    pub async fn run(&self, ctx: &CoreContext<'_>, username: &str) -> Result<Vec<Repository>> {
        let rights = ctx
            .db_service
            .external_account_rights_list(username)
            .await?;

        let mut repositories = Vec::new();
        if !rights.is_empty() {
            for right in rights {
                repositories.push(
                    ctx.db_service
                        .repositories_get_from_id(right.repository_id)
                        .await?
                        .unwrap(),
                );
            }
        }

        Ok(repositories)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use prbot_database_interface::DbService;
    use prbot_models::{ExternalAccount, ExternalAccountRight, Repository};

    use super::ListExternalAccountRights;
    use crate::context::tests::CoreContextTest;

    #[tokio::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let ctx = CoreContextTest::new();

        let repo1 = ctx
            .db_service
            .repositories_create(Repository {
                owner: "owner".into(),
                name: "name".into(),
                ..Default::default()
            })
            .await?;

        let repo2 = ctx
            .db_service
            .repositories_create(Repository {
                owner: "owner".into(),
                name: "name2".into(),
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
                repository_id: repo1.id,
                username: "acc".into(),
            })
            .await?;
        ctx.db_service
            .external_account_rights_create(ExternalAccountRight {
                repository_id: repo2.id,
                username: "acc".into(),
            })
            .await?;

        assert_eq!(
            ListExternalAccountRights
                .run(&ctx.as_context(), "acc")
                .await?,
            vec![repo1, repo2]
        );

        Ok(())
    }
}
