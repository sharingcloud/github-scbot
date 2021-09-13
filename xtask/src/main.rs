use argh::FromArgs;
use tasks::Tasks;

mod common;
mod tasks;

/// Args
#[derive(FromArgs, Debug)]
struct Args {
    #[argh(subcommand)]
    tasks: Tasks,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = argh::from_env();
    match args.tasks {
        Tasks::Lint(cmd) => {
            cmd.handle()?;
        }
        Tasks::Format(cmd) => {
            cmd.handle()?;
        }
        Tasks::Server(cmd) => {
            cmd.handle()?;
        }
        Tasks::Test(cmd) => {
            cmd.handle()?;
        }
        Tasks::SetVersion(cmd) => {
            cmd.handle()?;
        }
        Tasks::BuildImage(cmd) => {
            cmd.handle()?;
        }
        Tasks::TagImage(cmd) => {
            cmd.handle()?;
        }
        Tasks::PushImage(cmd) => {
            cmd.handle()?;
        }
    }

    Ok(())
}
