use std::fs;

use argh::FromArgs;
use cargo_toml::Manifest;

use crate::common::project_root;

/// set version
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "set-version")]
pub(crate) struct SetVersionTask {
    /// version
    #[argh(positional)]
    version: String,
}

impl SetVersionTask {
    pub fn handle(self) -> Result<(), Box<dyn std::error::Error>> {
        // Loop on all crates starting with github_scbot
        let crates_dir = project_root().join("crates");

        let toml_files: Vec<_> = fs::read_dir(crates_dir)?
            .filter_map(|folder| {
                let folder = folder.unwrap();
                if folder
                    .file_name()
                    .to_str()
                    .unwrap()
                    .starts_with("github_scbot")
                {
                    let toml = folder.path().join("Cargo.toml");
                    Some(toml)
                } else {
                    None
                }
            })
            .collect();

        for toml_file in &toml_files {
            let mut manifest = Manifest::from_path(toml_file)?;
            if let Some(mut package) = manifest.package {
                package.version = self.version.clone();
                manifest.package = Some(package);
            }

            let serialized = toml::to_string(&manifest)?;
            fs::write(toml_file, serialized)?;
        }

        Ok(())
    }
}
