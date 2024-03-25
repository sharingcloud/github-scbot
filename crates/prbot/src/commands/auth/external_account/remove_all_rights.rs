use clap::Parser;
use prbot_core::use_cases::auth::RemoveAllExternalAccountRights;

use crate::{commands::CommandContext, Result};

/// Remove all rights from account
#[derive(Parser)]
pub(crate) struct AuthExternalAccountRemoveAllRightsCommand {
    /// Account username
    pub username: String,
}

impl AuthExternalAccountRemoveAllRightsCommand {
    pub async fn run(self, ctx: CommandContext) -> Result<()> {
        RemoveAllExternalAccountRights
            .run(&ctx.as_core_context(), &self.username)
            .await?;

        writeln!(
            ctx.writer.write().await,
            "All rights removed from external account '{}'.",
            self.username
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use prbot_database_interface::DbService;
    use prbot_models::{ExternalAccount, ExternalAccountRight, Repository};

    use crate::testutils::{test_command, CommandContextTest};

    #[tokio::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let ctx = CommandContextTest::new();
        ctx.db_service
            .external_accounts_create(ExternalAccount {
                username: "me".into(),
                ..Default::default()
            })
            .await?;

        let repo = ctx
            .db_service
            .repositories_create(Repository {
                owner: "owner".into(),
                name: "name".into(),
                ..Default::default()
            })
            .await?;

        ctx.db_service
            .external_account_rights_create(ExternalAccountRight {
                repository_id: repo.id,
                username: "me".into(),
            })
            .await?;

        assert_eq!(
            test_command(
                ctx,
                &["auth", "external-accounts", "remove-all-rights", "me"]
            )
            .await,
            "All rights removed from external account 'me'.\n"
        );

        Ok(())
    }
}
