use std::{
    fs::File,
    io::{BufReader, Write},
    path::PathBuf,
};

use async_trait::async_trait;
use clap::Parser;
use github_scbot_database_interface::Exchanger;

use super::{Command, CommandContext};
use crate::Result;

/// Import all data
#[derive(Parser)]
pub(crate) struct ImportCommand {
    /// Tnput file
    input_file: PathBuf,
}

#[async_trait(?Send)]
impl Command for ImportCommand {
    async fn execute<W: Write>(self, ctx: CommandContext<W>) -> Result<()> {
        let file = File::open(&self.input_file)?;
        let reader = BufReader::new(file);
        Exchanger::import_from_json(ctx.db_service.as_ref(), reader).await?;

        Ok(())
    }
}
