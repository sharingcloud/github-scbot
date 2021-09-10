use std::path::{Path, PathBuf};

use argh::FromArgs;
use cargo_toml::Manifest;

/// Args
#[derive(FromArgs, Debug)]
struct Args {
    #[argh(subcommand)]
    tasks: Tasks,
}

/// Tasks
#[derive(FromArgs, Debug)]
#[argh(subcommand)]
enum Tasks {
    Format(FormatTask),
    Lint(LintTask),
    Server(ServerTask),
    SetVersion(SetVersionTask),
    BuildImage(BuildImageTask),
    TagImage(TagImageTask),
    PushImage(PushImageTask),
}

/// format all
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "fmt")]
struct FormatTask {
    /// error on changes
    #[argh(switch, short = 'e')]
    error: bool,
}

/// lint all
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "lint")]
struct LintTask {
    /// error on warnings
    #[argh(switch, short = 'e')]
    error: bool,
}

/// start server
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "server")]
struct ServerTask {
    /// development mode (watch)
    #[argh(switch, short = 'd')]
    dev: bool,

    /// trace messages
    #[argh(switch, short = 't')]
    trace: bool,
}

/// set version
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "set-version")]
struct SetVersionTask {
    /// version
    #[argh(positional)]
    version: String,
}

/// build image
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "build-image")]
struct BuildImageTask {
    /// version
    #[argh(option)]
    version: Option<String>,
}

/// tag image
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "tag-image")]
struct TagImageTask {
    /// version (or latest)
    #[argh(option)]
    version: Option<String>,
}

/// push image
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "push-image")]
struct PushImageTask {
    /// registry
    #[argh(positional)]
    registry: String,

    /// version
    #[argh(option)]
    version: Option<String>,
}

fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}

fn get_version() -> String {
    let root = project_root();
    let cli_toml = root
        .join("crates")
        .join("github_scbot_cli")
        .join("Cargo.toml");
    let manifest = Manifest::from_path(cli_toml).expect("Cargo.toml should exist");
    manifest.package.expect("should have package").version
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = argh::from_env();
    match args.tasks {
        Tasks::Lint(cmd) => {
            if cmd.error {
                duct::cmd!(
                    "cargo",
                    "clippy",
                    "--all-features",
                    "--all",
                    "--tests",
                    "--",
                    "-D",
                    "warnings"
                )
                .run()?;
            } else {
                duct::cmd!("cargo", "clippy", "--all-features", "--all", "--tests").run()?;
            }
        }
        Tasks::Format(cmd) => {
            if cmd.error {
                duct::cmd!("cargo", "fmt", "--all", "--", "--check").run()?;
            } else {
                duct::cmd!("cargo", "fmt", "--all").run()?;
            }
        }
        Tasks::Server(cmd) => {
            let current_env_log = std::env::var("RUST_LOG").unwrap_or_default();

            if cmd.trace {
                std::env::set_var("RUST_LOG", "info,github_scbot=trace");
            }

            if cmd.dev {
                duct::cmd!("cargo", "watch", "-x", "run-cli -- server").run()?;
            } else {
                duct::cmd!("cargo", "run-cli", "--", "server").run()?;
            }

            std::env::set_var("RUST_LOG", current_env_log);
        }
        Tasks::SetVersion(cmd) => {
            duct::cmd!("just", "set-version", cmd.version).run()?;
        }
        Tasks::BuildImage(cmd) => {
            let version = cmd.version.unwrap_or_else(get_version);

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
        }
        Tasks::TagImage(cmd) => {
            let current_version = get_version();
            let version = cmd.version.unwrap_or_else(|| "latest".to_string());

            duct::cmd!(
                "docker",
                "tag",
                format!("github-scbot:{}", current_version),
                format!("github-scbot:{}", version)
            )
            .run()?;
        }
        Tasks::PushImage(cmd) => {
            let version = cmd.version.unwrap_or_else(get_version);

            duct::cmd!(
                "docker",
                "push",
                format!("{}/github-scbot:{}", cmd.registry, version),
            )
            .run()?;
        }
    }

    Ok(())
}
