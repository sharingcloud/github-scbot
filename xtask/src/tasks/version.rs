use std::fs;

use argh::FromArgs;
use regex::{Captures, Regex};

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
    fn replace_crates_version(&self) -> Result<(), Box<dyn std::error::Error>> {
        let version_rgx = Regex::new("version = .*\n")?;

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
            let contents = fs::read_to_string(toml_file)?;
            let replaced = version_rgx.replace(&contents, |_caps: &Captures| {
                format!("version = \"{}\"\n", self.version)
            });

            fs::write(toml_file, replaced.as_bytes())?;
        }

        Ok(())
    }

    fn replace_compose_version(&self) -> Result<(), Box<dyn std::error::Error>> {
        let version_rgx = Regex::new("image: github-scbot:.*")?;
        let compose_path = project_root().join("docker").join("docker-compose.yml");
        let contents = fs::read_to_string(&compose_path)?;
        let replaced = version_rgx.replace(&contents, |_caps: &Captures| {
            format!("image: github-scbot:{}", self.version)
        });

        fs::write(compose_path, replaced.as_bytes())?;

        Ok(())
    }

    pub fn handle(self) -> Result<(), Box<dyn std::error::Error>> {
        self.replace_crates_version()?;
        self.replace_compose_version()
    }
}
