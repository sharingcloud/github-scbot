//! Auth commands.

use std::io::Write;

use async_trait::async_trait;
use clap::{Parser, Subcommand};

use self::{admin::AuthAdminCommand, external_account::AuthExternalAccountCommand};
use super::{Command, CommandContext};
use crate::Result;

mod admin;
mod external_account;

/// Authentication related commands
#[derive(Parser)]
pub(crate) struct AuthCommand {
    #[clap(subcommand)]
    inner: AuthSubCommand,
}

#[async_trait(?Send)]
impl Command for AuthCommand {
    async fn execute<W: Write>(self, ctx: CommandContext<W>) -> Result<()> {
        self.inner.execute(ctx).await
    }
}

#[derive(Subcommand)]
enum AuthSubCommand {
    Admins(AuthAdminCommand),
    ExternalAccounts(AuthExternalAccountCommand),
}

#[async_trait(?Send)]
impl Command for AuthSubCommand {
    async fn execute<W: Write>(self, ctx: CommandContext<W>) -> Result<()> {
        match self {
            Self::Admins(sub) => sub.execute(ctx).await,
            Self::ExternalAccounts(sub) => sub.execute(ctx).await,
        }
    }
}
