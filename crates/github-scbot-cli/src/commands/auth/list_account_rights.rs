use std::io::Write;

use crate::Result;
use clap::Parser;
use github_scbot_domain::use_cases::auth::ListAccountRightsUseCaseInterface;

/// List rights for account
#[derive(Parser)]
pub(crate) struct AuthListAccountRightsCommand {
    /// Account username
    pub username: String,
}

impl AuthListAccountRightsCommand {
    pub async fn run<W: Write>(
        self,
        mut writer: W,
        use_case: &dyn ListAccountRightsUseCaseInterface,
    ) -> Result<()> {
        let repositories = use_case.run().await?;

        if repositories.is_empty() {
            writeln!(writer, "No right found from account '{}'.", self.username)?;
        } else {
            writeln!(writer, "Rights from account '{}':", self.username)?;
            for repo in repositories {
                writeln!(writer, "- {}/{}", repo.owner(), repo.name())?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use github_scbot_database::Repository;
    use github_scbot_domain::DomainError;

    use super::*;
    use crate::testutils::buffer_to_string;

    #[actix_rt::test]
    async fn test_no_right() -> Result<()> {
        struct Impl;

        #[async_trait(?Send)]
        impl ListAccountRightsUseCaseInterface for Impl {
            async fn run(&self) -> Result<Vec<Repository>, DomainError> {
                Ok(vec![])
            }
        }

        let mut buf = Vec::new();
        let cmd = AuthListAccountRightsCommand {
            username: "me".into(),
        };
        cmd.run(&mut buf, &Impl).await?;
        assert_eq!(buffer_to_string(buf), "No right found from account 'me'.\n");

        Ok(())
    }

    #[actix_rt::test]
    async fn test_rights() -> Result<()> {
        struct Impl;

        #[async_trait(?Send)]
        impl ListAccountRightsUseCaseInterface for Impl {
            async fn run(&self) -> Result<Vec<Repository>, DomainError> {
                Ok(vec![
                    Repository::builder()
                        .owner("owner")
                        .name("name")
                        .build()
                        .unwrap(),
                    Repository::builder()
                        .owner("owner")
                        .name("name2")
                        .build()
                        .unwrap(),
                ])
            }
        }

        let mut buf = Vec::new();
        let cmd = AuthListAccountRightsCommand {
            username: "me".into(),
        };
        cmd.run(&mut buf, &Impl).await?;
        assert_eq!(
            buffer_to_string(buf),
            indoc::indoc! {r#"
                Rights from account 'me':
                - owner/name
                - owner/name2
            "#}
        );

        Ok(())
    }
}
