use crate::{CoreContext, Result};

pub struct GenerateExternalAccountToken;

impl GenerateExternalAccountToken {
    #[tracing::instrument(skip(self, ctx), fields(username))]
    pub async fn run(&self, ctx: &CoreContext<'_>, username: &str) -> Result<String> {
        let exa = ctx
            .db_service
            .external_accounts_get(username)
            .await?
            .unwrap();
        exa.generate_access_token().map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use prbot_database_interface::DbService;
    use prbot_models::ExternalAccount;

    use super::GenerateExternalAccountToken;
    use crate::context::tests::CoreContextTest;

    #[tokio::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let ctx = CoreContextTest::new();

        ctx.db_service
            .external_accounts_create(
                ExternalAccount {
                    username: "me".into(),
                    ..Default::default()
                }
                .with_generated_keys(),
            )
            .await?;

        assert!(GenerateExternalAccountToken
            .run(&ctx.as_context(), "me")
            .await?
            .starts_with("ey"));

        Ok(())
    }
}
