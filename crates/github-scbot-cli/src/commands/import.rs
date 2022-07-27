use std::{
    fs::File,
    io::{BufReader, Write},
    path::PathBuf,
};

use crate::errors::{DatabaseSnafu, IoSnafu};
use crate::Result;
use async_trait::async_trait;
use clap::Parser;
use github_scbot_database::Exchanger;
use snafu::ResultExt;

use super::{Command, CommandContext};

/// Import all data
#[derive(Parser)]
pub(crate) struct ImportCommand {
    /// Tnput file
    input_file: PathBuf,
}

#[async_trait(?Send)]
impl Command for ImportCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let file = File::open(&self.input_file).context(IoSnafu)?;
        let reader = BufReader::new(file);
        Exchanger::import_from_json(&mut *ctx.db_adapter, reader)
            .await
            .context(DatabaseSnafu)?;

        Ok(())
    }
}
