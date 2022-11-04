use std::io::Write;

use crate::Result;
use clap::Parser;
use github_scbot_domain::use_cases::auth::ListAdminAccountsUseCaseInterface;

/// List admin accounts
#[derive(Parser)]
pub(crate) struct AuthListAdminAccountsCommand;

impl AuthListAdminAccountsCommand {
    pub async fn run<W: Write>(
        self,
        mut writer: W,
        use_case: &dyn ListAdminAccountsUseCaseInterface,
    ) -> Result<()> {
        let accounts = use_case.run().await?;

        if accounts.is_empty() {
            writeln!(writer, "No admin account found.")?;
        } else {
            writeln!(writer, "Admin accounts:")?;
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
    use github_scbot_database::Account;
    use github_scbot_domain::DomainError;

    use super::*;
    use crate::testutils::buffer_to_string;

    #[actix_rt::test]
    async fn test_no_account() -> Result<()> {
        struct Impl;

        #[async_trait(?Send)]
        impl ListAdminAccountsUseCaseInterface for Impl {
            async fn run(&self) -> Result<Vec<Account>, DomainError> {
                Ok(vec![])
            }
        }

        let mut buf = Vec::new();
        let cmd = AuthListAdminAccountsCommand;
        cmd.run(&mut buf, &Impl).await?;
        assert_eq!(buffer_to_string(buf), "No admin account found.\n");

        Ok(())
    }

    #[actix_rt::test]
    async fn test_accounts() -> Result<()> {
        struct Impl;

        #[async_trait(?Send)]
        impl ListAdminAccountsUseCaseInterface for Impl {
            async fn run(&self) -> Result<Vec<Account>, DomainError> {
                Ok(vec![
                    Account::builder().username("one").build().unwrap(),
                    Account::builder().username("two").build().unwrap(),
                ])
            }
        }

        let mut buf = Vec::new();
        let cmd = AuthListAdminAccountsCommand;
        cmd.run(&mut buf, &Impl).await?;
        assert_eq!(
            buffer_to_string(buf),
            indoc::indoc! {r#"
                Admin accounts:
                - one
                - two
            "#}
        );

        Ok(())
    }
}
