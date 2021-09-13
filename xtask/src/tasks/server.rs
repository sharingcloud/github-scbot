use argh::FromArgs;

/// start server
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "server")]
pub(crate) struct ServerTask {
    /// development mode (watch)
    #[argh(switch, short = 'd')]
    dev: bool,

    /// trace messages
    #[argh(switch, short = 't')]
    trace: bool,
}

impl ServerTask {
    pub fn handle(self) -> Result<(), Box<dyn std::error::Error>> {
        let current_env_log = std::env::var("RUST_LOG").unwrap_or_default();

        if self.trace {
            std::env::set_var("RUST_LOG", "info,github_scbot=trace");
        }

        if self.dev {
            duct::cmd!("cargo", "watch", "-x", "run-cli -- server").run()?;
        } else {
            duct::cmd!("cargo", "run-cli", "--", "server").run()?;
        }

        std::env::set_var("RUST_LOG", current_env_log);
        Ok(())
    }
}
