use crate::Result;
use clap::Parser;
use github_scbot_core::config::Config;
use github_scbot_database::{establish_pool_connection, run_migrations, DbServiceImplPool};
use github_scbot_server::{ghapi::MetricsApiService, redis::MetricsRedisService};
use std::io::Write;

use crate::commands::{Command, CommandContext, SubCommand};

/// SharingCloud PR Bot
#[derive(Parser)]
#[clap(author, version, about, long_about = None, name = "github-scbot")]
#[clap(propagate_version = true)]
pub struct Args {
    #[clap(subcommand)]
    cmd: SubCommand,
}

pub struct CommandExecutor;

impl CommandExecutor {
    pub fn parse_args(config: Config, args: Args) -> Result<()> {
        let sync = |config: Config, args: Args| async {
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

            Self::parse_args_async(args, ctx).await
        };

        actix_rt::System::with_tokio_rt(|| {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
        })
        .block_on(sync(config, args))?;

        Ok(())
    }

    pub(crate) async fn parse_args_async<W: Write>(
        args: Args,
        ctx: CommandContext<W>,
    ) -> Result<()> {
        args.cmd.execute(ctx).await
    }
}
