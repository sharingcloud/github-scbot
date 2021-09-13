use argh::FromArgs;

use crate::common::get_version;

/// build image
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "build-image")]
pub(crate) struct BuildImageTask {
    /// version
    #[argh(option)]
    version: Option<String>,
}

impl BuildImageTask {
    pub fn handle(self) -> Result<(), Box<dyn std::error::Error>> {
        let version = self.version.unwrap_or_else(get_version);

        duct::cmd!(
            "docker",
            "build",
            "--rm",
            "-t",
            format!("github-scbot:{}", version),
            "-f",
            "./docker/Dockerfile",
            "."
        )
        .run()?;

        Ok(())
    }
}

/// tag image
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "tag-image")]
pub(crate) struct TagImageTask {
    /// version (or latest)
    #[argh(option)]
    version: Option<String>,
}

impl TagImageTask {
    pub fn handle(self) -> Result<(), Box<dyn std::error::Error>> {
        let current_version = get_version();
        let version = self.version.unwrap_or_else(|| "latest".to_string());

        duct::cmd!(
            "docker",
            "tag",
            format!("github-scbot:{}", current_version),
            format!("github-scbot:{}", version)
        )
        .run()?;

        Ok(())
    }
}

/// push image
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "push-image")]
pub(crate) struct PushImageTask {
    /// registry
    #[argh(positional)]
    registry: String,

    /// version
    #[argh(option)]
    version: Option<String>,
}

impl PushImageTask {
    pub fn handle(self) -> Result<(), Box<dyn std::error::Error>> {
        let version = self.version.unwrap_or_else(get_version);

        duct::cmd!(
            "docker",
            "push",
            format!("{}/github-scbot:{}", self.registry, version),
        )
        .run()?;

        Ok(())
    }
}
