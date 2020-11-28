//! SC Bot entrypoint

use color_eyre::eyre::Result;
use github_scbot::{configure_startup, run_bot_server};

fn main() -> Result<()> {
    configure_startup()?;

    // Run bot
    run_bot_server()
}
