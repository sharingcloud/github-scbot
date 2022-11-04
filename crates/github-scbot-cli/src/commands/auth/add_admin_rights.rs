use std::io::Write;

use crate::Result;
use clap::Parser;
use github_scbot_domain::use_cases::auth::AddAdminRightUseCaseInterface;

/// Add admin rights to account
#[derive(Parser)]
pub(crate) struct AuthAddAdminRightsCommand {
    /// Account username
    pub username: String,
}

impl AuthAddAdminRightsCommand {
    pub async fn run<W: Write>(
        self,
        mut writer: W,
        use_case: &dyn AddAdminRightUseCaseInterface,
    ) -> Result<()> {
        use_case.run().await?;

        writeln!(
            writer,
            "Account '{}' added/edited with admin rights.",
            self.username
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use github_scbot_domain::DomainError;

    use super::*;
    use crate::testutils::buffer_to_string;

    struct Impl;

    #[async_trait(?Send)]
    impl AddAdminRightUseCaseInterface for Impl {
        async fn run(&self) -> Result<(), DomainError> {
            Ok(())
        }
    }

    #[actix_rt::test]
    async fn test() -> Result<()> {
        let mut buf = Vec::new();
        let cmd = AuthAddAdminRightsCommand {
            username: "me".into(),
        };
        cmd.run(&mut buf, &Impl).await?;

        assert_eq!(
            buffer_to_string(buf),
            "Account 'me' added/edited with admin rights.\n"
        );

        Ok(())
    }
}
