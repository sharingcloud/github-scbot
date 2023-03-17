use std::io::Write;

use clap::Parser;
use github_scbot_config::Config;
use github_scbot_database_pg::{establish_pool_connection, run_migrations, PostgresDb};
use github_scbot_server::{ghapi::MetricsApiService, redis::MetricsRedisService};

use crate::{
    commands::{Command, CommandContext, SubCommand},
    Result,
};

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

            let db_service = PostgresDb::new(pool);
            let api_service = MetricsApiService::new(config.clone());
            let lock_service = MetricsRedisService::new(&config.redis_address);
            let ctx = CommandContext {
                config,
                db_service: Box::new(db_service),
                api_service: Box::new(api_service),
                lock_service: Box::new(lock_service),
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
