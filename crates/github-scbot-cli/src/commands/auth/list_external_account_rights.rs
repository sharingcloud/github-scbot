use std::io::Write;

use crate::{commands::CommandContext, Result};
use clap::Parser;
use github_scbot_domain::use_cases::auth::ListExternalAccountRightsUseCase;

/// List rights for account
#[derive(Parser)]
pub(crate) struct AuthListExternalAccountRightsCommand {
    /// Account username
    pub username: String,
}

impl AuthListExternalAccountRightsCommand {
    pub async fn run<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let repositories = ListExternalAccountRightsUseCase {
            username: &self.username,
            db_service: ctx.db_adapter.as_mut(),
        }
        .run()
        .await?;

        if repositories.is_empty() {
            writeln!(
                ctx.writer,
                "No right found from external account '{}'.",
                self.username
            )?;
        } else {
            writeln!(
                ctx.writer,
                "Rights from external account '{}':",
                self.username
            )?;
            for repo in repositories {
                writeln!(ctx.writer, "- {}/{}", repo.owner, repo.name)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use github_scbot_database::{DbService, ExternalAccount, ExternalAccountRight, Repository};

    use crate::testutils::{test_command, CommandContextTest};

    #[actix_rt::test]
    async fn run_no_rights() -> Result<(), Box<dyn Error>> {
        let ctx = CommandContextTest::new();

        assert_eq!(
            test_command(ctx, &["auth", "list-external-account-rights", "me"]).await,
            "No right found from external account 'me'.\n"
        );

        Ok(())
    }

    #[actix_rt::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let mut ctx = CommandContextTest::new();
        ctx.db_adapter
            .external_accounts_create(ExternalAccount {
                username: "me".into(),
                ..Default::default()
            })
            .await?;

        let repo = ctx
            .db_adapter
            .repositories_create(Repository {
                owner: "owner".into(),
                name: "name".into(),
                ..Default::default()
            })
            .await?;

        ctx.db_adapter
            .external_account_rights_create(ExternalAccountRight {
                repository_id: repo.id,
                username: "me".into(),
            })
            .await?;

        assert_eq!(
            test_command(ctx, &["auth", "list-external-account-rights", "me"]).await,
            "Rights from external account 'me':\n- owner/name\n"
        );

        Ok(())
    }
}
