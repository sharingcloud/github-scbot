//! CLI module.

use anyhow::Result;
use std::io::Write;

use clap::Parser;
use github_scbot_core::config::{configure_startup, Config};
use github_scbot_database::{establish_pool_connection, run_migrations, DbServiceImplPool};
use github_scbot_server::{ghapi::MetricsApiService, redis::MetricsRedisService};

use self::commands::{Command, CommandContext, SubCommand};

mod commands;
pub(crate) mod utils;

/// SharingCloud PR Bot
#[derive(Parser)]
#[clap(author, version, about, long_about = None, name = "github-scbot")]
#[clap(propagate_version = true)]
struct Args {
    #[clap(subcommand)]
    cmd: SubCommand,
}

/// Initialize command line.
pub fn initialize_command_line() -> Result<()> {
    let config = configure_startup()?;
    let args = Args::parse();

    parse_args_sync(config, args)
}

fn parse_args_sync(config: Config, args: Args) -> Result<()> {
    async fn sync(config: Config, args: Args) -> Result<()> {
        let pool = establish_pool_connection(&config).await?;
        run_migrations(&pool).await?;

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

    actix_rt::System::with_tokio_rt(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
    })
    .block_on(sync(config, args))?;

    Ok(())
}

async fn parse_args<W: Write>(args: Args, ctx: CommandContext<W>) -> Result<()> {
    args.cmd.execute(ctx).await
}

#[cfg(test)]
mod testutils {
    use super::Result;
    use clap::Parser;
    use github_scbot_core::config::Config;
    use github_scbot_database::DbService;
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

            let command_args = {
                let mut tmp_args = vec!["bot"];
                tmp_args.extend(command_args);
                tmp_args
            };

            let args = Args::try_parse_from(command_args).unwrap();
            parse_args(args, ctx).await?;
        }

        Ok(std::str::from_utf8(buf.as_slice()).unwrap().to_string())
    }
}
