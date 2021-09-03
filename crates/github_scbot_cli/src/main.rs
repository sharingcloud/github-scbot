//! Entrypoint.

use github_scbot_libs::owo_colors::OwoColorize;

fn main() {
    if let Err(err) = github_scbot_cli::initialize_command_line() {
        eprintln!("{}", format!("ERROR: {:?}", err).red());
        std::process::exit(1);
    }
}
