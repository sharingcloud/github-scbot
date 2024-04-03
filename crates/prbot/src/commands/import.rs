use std::{fs::File, io::BufReader, path::PathBuf};

use async_trait::async_trait;
use clap::Parser;
use prbot_database_interface::Exchanger;

use super::{Command, CommandContext};
use crate::Result;

/// Import all data
#[derive(Parser)]
pub(crate) struct ImportCommand {
    /// Tnput file
    input_file: PathBuf,
}

#[async_trait]
impl Command for ImportCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let file = File::open(&self.input_file)?;
        let reader = BufReader::new(file);
        Exchanger::import_from_json(ctx.db_service.as_ref(), reader).await?;

        Ok(())
    }
}
