use argh::FromArgs;

/// format all
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "fmt")]
pub(crate) struct FormatTask {
    /// error on changes
    #[argh(switch, short = 'e')]
    error: bool,
}

impl FormatTask {
    pub fn handle(self) -> Result<(), Box<dyn std::error::Error>> {
        if self.error {
            duct::cmd!("cargo", "fmt", "--all", "--", "--check").run()?;
        } else {
            duct::cmd!("cargo", "fmt", "--all").run()?;
        }

        Ok(())
    }
}
