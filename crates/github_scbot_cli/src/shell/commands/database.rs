//! Database commands.

use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

use github_scbot_conf::Config;
use github_scbot_database::{
    import_export::{export_models_to_json, import_models_from_json, ExportError, ImportError},
    models::IDatabaseAdapter,
};

use super::errors::Result;

pub(crate) async fn export_json(
    db_adapter: &dyn IDatabaseAdapter,
    output_path: Option<PathBuf>,
) -> Result<()> {
    if let Some(file_path) = output_path {
        let file = File::create(file_path.clone())
            .map_err(|e| ExportError::IoError(file_path, e.to_string()))?;
        let mut writer = BufWriter::new(file);
        export_models_to_json(db_adapter, &mut writer).await?;
    } else {
        let mut writer = std::io::stdout();
        export_models_to_json(db_adapter, &mut writer).await?;
    }

    Ok(())
}

pub(crate) async fn import_json(
    config: &Config,
    db_adapter: &dyn IDatabaseAdapter,
    input_path: &Path,
) -> Result<()> {
    let file = File::open(input_path.to_path_buf())
        .map_err(|e| ImportError::IoError(input_path.to_path_buf(), e.to_string()))?;
    let reader = BufReader::new(file);
    import_models_from_json(config, db_adapter, reader).await?;

    Ok(())
}
