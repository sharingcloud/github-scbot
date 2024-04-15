use clap::Parser;
use prbot_server::admin::validator::generate_admin_token;

use crate::{commands::CommandContext, Result};

/// Create admin token
#[derive(Parser)]
pub(crate) struct GenerateTokenCommand;

impl GenerateTokenCommand {
    pub async fn run(self, ctx: CommandContext) -> Result<()> {
        let admin_token = generate_admin_token(&ctx.config)?;
        writeln!(ctx.writer.write().await, "{}", admin_token)?;
        Ok(())
    }
}
