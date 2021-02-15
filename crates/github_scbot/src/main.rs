//! Entrypoint.

use owo_colors::OwoColorize;

fn main() {
    if let Err(err) = github_scbot::initialize_command_line() {
        eprintln!("{}", format!("ERROR: {:?}", err).red());
        std::process::exit(1);
    }
}
