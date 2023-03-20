use std::io::Write;

use clap::Parser;
use github_scbot_domain::use_cases::auth::RemoveAllExternalAccountRightsUseCase;

use crate::{commands::CommandContext, Result};

/// Remove all rights from account
#[derive(Parser)]
pub(crate) struct AuthRemoveAllExternalAccountRightsCommand {
    /// Account username
    pub username: String,
}

impl AuthRemoveAllExternalAccountRightsCommand {
    pub async fn run<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        RemoveAllExternalAccountRightsUseCase {
            db_service: ctx.db_service.as_ref(),
        }
        .run(&self.username)
        .await?;

        writeln!(
            ctx.writer,
            "All rights removed from external account '{}'.",
            self.username
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use github_scbot_database_interface::DbService;
    use github_scbot_domain_models::{ExternalAccount, ExternalAccountRight, Repository};

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
            test_command(ctx, &["auth", "remove-all-external-account-rights", "me"]).await,
            "All rights removed from external account 'me'.\n"
        );

        Ok(())
    }
}
