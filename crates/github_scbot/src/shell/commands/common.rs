//! Common commands.

use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

use github_scbot_database::{
    establish_single_connection,
    import_export::{export_models_to_json, import_models_from_json, ExportError, ImportError},
    Result,
};

/// Export database as JSON.
///
/// # Arguments
///
/// * `output_path` - Optional output path.
pub fn export_json(output_path: Option<PathBuf>) -> Result<()> {
    let conn = establish_single_connection()?;

    if let Some(file_path) = output_path {
        let file =
            File::create(file_path.clone()).map_err(|e| ExportError::IOError(file_path, e))?;
        let mut writer = BufWriter::new(file);
        export_models_to_json(&conn, &mut writer)
    } else {
        let mut writer = std::io::stdout();
        export_models_to_json(&conn, &mut writer)
    }
}

/// Import database from JSON.
///
/// # Arguments
///
/// * `input_path` - Input path.
pub fn import_json(input_path: &Path) -> Result<()> {
    let conn = establish_single_connection()?;

    let file = File::open(input_path.to_path_buf())
        .map_err(|e| ImportError::IOError(input_path.to_path_buf(), e))?;
    let reader = BufReader::new(file);
    import_models_from_json(&conn, reader)?;

    Ok(())
}
