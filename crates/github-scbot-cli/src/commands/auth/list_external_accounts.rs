use std::io::Write;

use crate::Result;
use clap::Parser;
use github_scbot_domain::use_cases::auth::ListExternalAccountsUseCaseInterface;

/// List external accounts
#[derive(Parser)]
pub(crate) struct AuthListExternalAccountsCommand;

impl AuthListExternalAccountsCommand {
    pub async fn run<W: Write>(
        self,
        mut writer: W,
        use_case: &dyn ListExternalAccountsUseCaseInterface,
    ) -> Result<()> {
        let accounts = use_case.run().await?;

        if accounts.is_empty() {
            writeln!(writer, "No external account found.")?;
        } else {
            writeln!(writer, "External accounts:")?;
            for account in accounts {
                writeln!(writer, "- {}", account.username())?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use github_scbot_database::ExternalAccount;
    use github_scbot_domain::DomainError;

    use super::*;
    use crate::testutils::buffer_to_string;

    #[actix_rt::test]
    async fn test_no_account() -> Result<()> {
        struct Impl;

        #[async_trait(?Send)]
        impl ListExternalAccountsUseCaseInterface for Impl {
            async fn run(&self) -> Result<Vec<ExternalAccount>, DomainError> {
                Ok(vec![])
            }
        }

        let mut buf = Vec::new();
        let cmd = AuthListExternalAccountsCommand;
        cmd.run(&mut buf, &Impl).await?;
        assert_eq!(buffer_to_string(buf), "No external account found.\n");

        Ok(())
    }

    #[actix_rt::test]
    async fn test_accounts() -> Result<()> {
        struct Impl;

        #[async_trait(?Send)]
        impl ListExternalAccountsUseCaseInterface for Impl {
            async fn run(&self) -> Result<Vec<ExternalAccount>, DomainError> {
                Ok(vec![
                    ExternalAccount::builder().username("one").build().unwrap(),
                    ExternalAccount::builder().username("two").build().unwrap(),
                ])
            }
        }

        let mut buf = Vec::new();
        let cmd = AuthListExternalAccountsCommand;
        cmd.run(&mut buf, &Impl).await?;
        assert_eq!(
            buffer_to_string(buf),
            indoc::indoc! {r#"
                External accounts:
                - one
                - two
            "#}
        );

        Ok(())
    }
}
