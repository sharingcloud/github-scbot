use clap::Parser;
use prbot_core::use_cases::auth::ListExternalAccountRights;

use crate::{commands::CommandContext, Result};

/// List rights for account
#[derive(Parser)]
pub(crate) struct AuthExternalAccountListRightsCommand {
    /// Account username
    pub username: String,
}

impl AuthExternalAccountListRightsCommand {
    pub async fn run(self, ctx: CommandContext) -> Result<()> {
        let repositories = ListExternalAccountRights
            .run(&ctx.as_core_context(), &self.username)
            .await?;

        if repositories.is_empty() {
            writeln!(
                ctx.writer.write().await,
                "No right found from external account '{}'.",
                self.username
            )?;
        } else {
            writeln!(
                ctx.writer.write().await,
                "Rights from external account '{}':",
                self.username
            )?;
            for repo in repositories {
                writeln!(ctx.writer.write().await, "- {}/{}", repo.owner, repo.name)?;
            }
        }

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
    async fn run_no_rights() -> Result<(), Box<dyn Error>> {
        let ctx = CommandContextTest::new();

        assert_eq!(
            test_command(ctx, &["auth", "external-accounts", "list-rights", "me"]).await,
            "No right found from external account 'me'.\n"
        );

        Ok(())
    }

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
            test_command(ctx, &["auth", "external-accounts", "list-rights", "me"]).await,
            "Rights from external account 'me':\n- owner/name\n"
        );

        Ok(())
    }
}
