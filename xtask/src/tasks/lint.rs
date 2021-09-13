use argh::FromArgs;

/// lint all
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "lint")]
pub(crate) struct LintTask {
    /// error on warnings
    #[argh(switch, short = 'e')]
    error: bool,
}

impl LintTask {
    pub fn handle(self) -> Result<(), Box<dyn std::error::Error>> {
        if self.error {
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

        Ok(())
    }
}
