//! CLI module.

use std::{ffi::OsStr, path::Path};

use argh::FromArgs;
use github_scbot_conf::{configure_startup, Config};
use github_scbot_database2::{establish_pool_connection, run_migrations, DbServiceImplPool};
use github_scbot_ghapi::adapter::GithubApiService;
use github_scbot_redis::RedisServiceImpl;
use github_scbot_sentry::eyre::{self, eyre::eyre};

use self::commands::{Command, CommandContext, SubCommand};

mod commands;
pub(crate) mod utils;

/// command.
#[derive(FromArgs)]
#[argh(description = "SharingCloud PR Bot")]
struct Args {
    #[argh(subcommand)]
    cmd: Option<SubCommand>,

    /// do not ask for input.
    #[argh(switch)]
    no_input: bool,

    /// show version.
    #[argh(switch)]
    version: bool,
}

/// Initialize command line.
pub fn initialize_command_line() -> eyre::Result<()> {
    // Prepare startup
    let config = configure_startup()?;

    async fn sync(config: Config, cmd: SubCommand, no_input: bool) -> eyre::Result<()> {
        let pool = establish_pool_connection(&config).await?;
        run_migrations(&pool).await?;

        let db_adapter = DbServiceImplPool::new(pool);
        let api_adapter = GithubApiService::new(config.clone());
        let redis_adapter = RedisServiceImpl::new(&config.redis_address);
        let ctx = CommandContext {
            config,
            db_adapter: Box::new(db_adapter),
            api_adapter: Box::new(api_adapter),
            redis_adapter: Box::new(redis_adapter),
            no_input,
        };

        cmd.execute(ctx).await
    }

    let args: Args = argh::from_env();
    if args.version {
        let exec_name = std::env::args()
            .next()
            .as_ref()
            .map(Path::new)
            .and_then(Path::file_name)
            .and_then(OsStr::to_str)
            .map(String::from)
            .unwrap();
        let version = env!("CARGO_PKG_VERSION");
        println!("{} {}", exec_name, version)
    } else if let Some(cmd) = args.cmd {
        actix_rt::System::with_tokio_rt(|| {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
        })
        .block_on(sync(config, cmd, args.no_input))?;
    } else {
        return Err(eyre!("Missing subcommand. Use --help for more info."));
    }

    Ok(())
}
