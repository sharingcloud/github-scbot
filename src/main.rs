//! Entrypoint.

use owo_colors::OwoColorize;

fn main() {
    if let Err(err) = github_scbot::initialize_command_line() {
        eprintln!("{}", format!("ERROR: {}", err).red());
        err.chain()
            .skip(1)
            .for_each(|cause| eprintln!("{}", format!("because: {}", cause).red()));
        std::process::exit(1);
    }
}
