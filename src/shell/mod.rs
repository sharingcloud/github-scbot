//! Shell module

use color_eyre::Result;
use structopt::StructOpt;

use crate::server::run_bot_server;

#[derive(StructOpt, Debug)]
enum Command {
    /// Start bot server
    Server,
}

#[derive(StructOpt, Debug)]
struct Opt {
    /// Activate verbose mode
    #[structopt(short, long)]
    verbose: bool,

    #[structopt(subcommand)]
    cmd: Command,
}

fn configure_startup() -> Result<()> {
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "info");

    pretty_env_logger::init();
    color_eyre::install()?;

    Ok(())
}

/// Initialize command line
///
/// # Errors
///
/// - Startup error
///
pub fn initialize_command_line() -> Result<()> {
    // Prepare startup
    configure_startup()?;

    let opt = Opt::from_args();
    match opt.cmd {
        Command::Server => {
            run_bot_server()?;
        }
    }

    Ok(())
}
