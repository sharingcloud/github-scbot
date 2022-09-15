//! Entrypoint.

use owo_colors::OwoColorize;
use snafu::ErrorCompat;

fn main() {
    if let Err(err) = github_scbot_cli::initialize_command_line() {
        eprintln!("{}", format!("ERROR: {}", err).red());
        if let Some(bt) = ErrorCompat::backtrace(&err) {
            eprintln!("{:#?}", bt);
        }

        std::process::exit(1);
    }
}
