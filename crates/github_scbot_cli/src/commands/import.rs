use std::{
    fs::File,
    io::{BufReader, Write},
    path::PathBuf,
};

use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_database2::Exchanger;
use github_scbot_sentry::eyre::Result;

use super::{Command, CommandContext};

/// import all data.
#[derive(FromArgs)]
#[argh(subcommand, name = "import")]
pub(crate) struct ImportCommand {
    /// input file.
    #[argh(positional)]
    input_file: PathBuf,
}

#[async_trait(?Send)]
impl Command for ImportCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let file = File::open(&self.input_file)?;
        let reader = BufReader::new(file);
        Exchanger::import_from_json(&mut *ctx.db_adapter, reader).await?;

        Ok(())
    }
}
