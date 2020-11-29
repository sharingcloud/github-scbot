//! SC Bot entrypoint

use color_eyre::Result;

fn main() -> Result<()> {
    github_scbot::initialize_command_line()
}
