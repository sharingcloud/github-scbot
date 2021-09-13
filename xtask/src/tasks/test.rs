use argh::FromArgs;

/// test
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "lint")]
pub(crate) struct TestTask {
    /// with coverage
    #[argh(switch, short = 'c')]
    coverage: bool,
}

impl TestTask {
    pub fn handle(self) -> Result<(), Box<dyn std::error::Error>> {
        if self.coverage {
            std::env::set_var("CARGO_INCREMENTAL", "0");
            std::env::set_var("RUSTFLAGS", "-Zprofile -Ccodegen-units=1 -Cinline-threshold=0 -Clink-dead-code -Coverflow-checks=off -Cpanic=abort -Zpanic_abort_tests");
            std::env::set_var("RUSTDOCFLAGS", "-Zprofile -Ccodegen-units=1 -Cinline-threshold=0 -Clink-dead-code -Coverflow-checks=off -Cpanic=abort -Zpanic_abort_tests");
            duct::cmd!("cargo", "test", "--all-features", "--no-fail-fast").run()?;
            duct::cmd!(
                "grcov",
                ".",
                "-s",
                ".",
                "--binary-path",
                "./target/debug",
                "-t",
                "html",
                "--branch",
                "--ignore-not-existing",
                "--ignore",
                "/*",
                "--ignore",
                "*/tests/*",
                "-o",
                "./target/debug/coverage/"
            )
            .run()?;
        } else {
            duct::cmd!("cargo", "test", "--lib").run()?;
        }

        Ok(())
    }
}
