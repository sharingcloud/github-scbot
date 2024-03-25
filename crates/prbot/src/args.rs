use std::sync::Arc;

use clap::Parser;
use prbot_config::{ApiDriver, Config, DatabaseDriver, LockDriver};
use prbot_core::CoreModule;
use prbot_database_interface::DbService;
use prbot_database_memory::MemoryDb;
use prbot_database_pg::{establish_pool_connection, run_migrations, PostgresDb};
use prbot_ghapi_interface::ApiService;
use prbot_ghapi_null::NullApiService;
use prbot_lock_interface::LockService;
use prbot_lock_null::NullLockService;
use prbot_sentry::with_sentry_configuration;
use prbot_server::{ghapi::MetricsApiService, redis::MetricsRedisService};
use tokio::sync::RwLock;
use tracing::info;

use crate::{
    commands::{Command, CommandContext, SubCommand},
    Result,
};

#[derive(Parser)]
#[command(about = None, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    cmd: SubCommand,
}

pub struct CommandExecutor;

impl CommandExecutor {
    pub fn parse_args(config: Config, args: Args) -> Result<()> {
        let sync = |config: Config, args: Args| async move {
            let core_module = CoreModule::builder().build();
            let db_service: Box<dyn DbService + Send + Sync + 'static> = {
                if config.database.driver == DatabaseDriver::Postgres {
                    info!("Using PostgresDb database driver");

                    let pool = establish_pool_connection(&config).await?;
                    run_migrations(&pool).await?;

                    Box::new(PostgresDb::new(pool))
                } else {
                    info!("Using MemoryDb database driver");
                    Box::new(MemoryDb::new())
                }
            };

            let api_service: Box<dyn ApiService + Send + Sync + 'static> = {
                if config.api.driver == ApiDriver::GitHub {
                    info!("Using MetricsApiService API driver");
                    Box::new(MetricsApiService::new(config.clone()))
                } else {
                    info!("Using NullApiService API driver");
                    Box::new(NullApiService::new())
                }
            };

            let lock_service: Box<dyn LockService + Send + Sync + 'static> = {
                if config.lock.driver == LockDriver::Redis {
                    info!("Using RedisLockService lock driver");
                    Box::new(MetricsRedisService::new(&config.lock.redis.address))
                } else {
                    info!("Using NullLockService lock driver");
                    Box::new(NullLockService::new())
                }
            };

            let ctx = CommandContext {
                config: config.clone(),
                db_service,
                api_service,
                lock_service,
                core_module,
                writer: Arc::new(RwLock::new(std::io::stdout())),
            };

            with_sentry_configuration(&config.clone(), || async {
                Self::parse_args_async(args, ctx).await
            })
            .await
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

    pub(crate) async fn parse_args_async(args: Args, ctx: CommandContext) -> Result<()> {
        args.cmd.execute(ctx).await
    }
}
