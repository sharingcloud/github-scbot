use std::{fs::File, io::BufWriter, path::PathBuf};

use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_database2::Exchanger;
use github_scbot_sentry::eyre::Result;

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
    async fn execute(self, mut ctx: CommandContext) -> Result<()> {
        if let Some(file_path) = self.output_file {
            let file = File::create(file_path.clone())?;
            let mut writer = BufWriter::new(file);
            Exchanger::export_to_json(&mut *ctx.db_adapter, &mut writer)
                .await
                .map_err(Into::into)
        } else {
            let mut writer = std::io::stdout();
            Exchanger::export_to_json(&mut *ctx.db_adapter, &mut writer)
                .await
                .map_err(Into::into)
        }
    }
}
