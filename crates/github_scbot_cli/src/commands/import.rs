use std::{fs::File, io::BufReader, path::PathBuf};

use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_database::import_export::{import_models_from_json, ImportError};
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
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        let file = File::open(&self.input_file)
            .map_err(|e| ImportError::IoError(self.input_file.to_path_buf(), e.to_string()))?;
        let reader = BufReader::new(file);
        import_models_from_json(&ctx.config, ctx.db_adapter.as_ref(), reader).await?;

        Ok(())
    }
}
