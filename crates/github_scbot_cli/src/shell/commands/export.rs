use std::{fs::File, io::BufWriter, path::PathBuf};

use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_database::import_export::{export_models_to_json, ExportError};
use stable_eyre::eyre::Result;

use super::{Command, CommandContext};

/// export all data.
#[derive(FromArgs)]
#[argh(subcommand, name = "export")]
pub(crate) struct ExportCommand {
    /// output file, stdout if not precised.
    #[argh(option, short = 'o')]
    output_file: Option<PathBuf>,
}

#[async_trait(?Send)]
impl Command for ExportCommand {
    async fn execute<'a>(self, ctx: CommandContext<'a>) -> Result<()> {
        if let Some(file_path) = self.output_file {
            let file = File::create(file_path.clone())
                .map_err(|e| ExportError::IoError(file_path, e.to_string()))?;
            let mut writer = BufWriter::new(file);
            export_models_to_json(ctx.db_adapter, &mut writer)
                .await
                .map_err(Into::into)
        } else {
            let mut writer = std::io::stdout();
            export_models_to_json(ctx.db_adapter, &mut writer)
                .await
                .map_err(Into::into)
        }
    }
}
