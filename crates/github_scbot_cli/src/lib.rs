//! CLI module.

use std::{ffi::OsStr, io::Write, path::Path};

use argh::FromArgs;
use errors::CliError;
use github_scbot_conf::{configure_startup, Config};
use github_scbot_database2::{establish_pool_connection, run_migrations, DbServiceImplPool};
use github_scbot_server::{ghapi::MetricsApiService, redis::MetricsRedisService};
use snafu::{whatever, ResultExt};

use self::commands::{Command, CommandContext, SubCommand};

mod commands;
pub(crate) mod errors;
pub(crate) mod utils;
use errors::{ConfSnafu, DatabaseSnafu};

type Result<T, E = CliError> = std::result::Result<T, E>;

/// command.
#[derive(FromArgs)]
#[argh(description = "SharingCloud PR Bot")]
struct Args {
    #[argh(subcommand)]
    cmd: Option<SubCommand>,

    /// show version.
    #[argh(switch)]
    version: bool,
}

/// Initialize command line.
pub fn initialize_command_line() -> Result<()> {
    let config = configure_startup().context(ConfSnafu)?;
    let args: Args = argh::from_env();

    parse_args_sync(config, args)
}

fn parse_args_sync(config: Config, args: Args) -> Result<()> {
    async fn sync(config: Config, args: Args) -> Result<()> {
        let pool = establish_pool_connection(&config)
            .await
            .context(DatabaseSnafu)?;
        run_migrations(&pool).await.context(DatabaseSnafu)?;

        let db_adapter = DbServiceImplPool::new(pool);
        let api_adapter = MetricsApiService::new(config.clone());
        let redis_adapter = MetricsRedisService::new(&config.redis_address);
        let ctx = CommandContext {
            config,
            db_adapter: Box::new(db_adapter),
            api_adapter: Box::new(api_adapter),
            redis_adapter: Box::new(redis_adapter),
            writer: Box::new(std::io::stdout()),
        };

        parse_args(args, ctx).await
    }

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
    } else {
        actix_rt::System::with_tokio_rt(|| {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
        })
        .block_on(sync(config, args))?;
    }

    Ok(())
}

async fn parse_args<W: Write>(args: Args, ctx: CommandContext<W>) -> Result<()> {
    if let Some(cmd) = args.cmd {
        cmd.execute(ctx).await
    } else {
        whatever!("Missing subcommand. Use --help for more info.");
    }
}

#[cfg(test)]
mod testutils {
    use super::Result;
    use argh::FromArgs;
    use github_scbot_conf::Config;
    use github_scbot_database2::DbService;
    use github_scbot_ghapi::adapter::ApiService;
    use github_scbot_redis::RedisService;

    use crate::{commands::CommandContext, parse_args, Args};

    pub(crate) async fn test_command(
        config: Config,
        db_adapter: Box<dyn DbService>,
        api_adapter: Box<dyn ApiService>,
        redis_adapter: Box<dyn RedisService>,
        command_args: &[&str],
    ) -> Result<String> {
        let mut buf = Vec::new();

        {
            let ctx = CommandContext {
                config,
                api_adapter,
                redis_adapter,
                db_adapter,
                writer: &mut buf,
            };

            let args = Args::from_args(&["bot"], command_args).unwrap();
            parse_args(args, ctx).await?;
        }

        Ok(std::str::from_utf8(buf.as_slice()).unwrap().to_string())
    }
}
