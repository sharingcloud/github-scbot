use std::{io::Write, sync::Arc};

use clap::Parser;
use prbot_config::Config;
use prbot_core::CoreModule;
use prbot_database_memory::MemoryDb;
use prbot_ghapi_interface::MockApiService;
use prbot_lock_interface::MockLockService;
use tokio::sync::RwLock;

use crate::{
    args::{Args, CommandExecutor},
    commands::CommandContext,
};

pub(crate) struct CommandContextTest {
    pub config: Config,
    pub core_module: CoreModule,
    pub api_service: MockApiService,
    pub db_service: MemoryDb,
    pub lock_service: MockLockService,
}

impl CommandContextTest {
    pub fn new() -> Self {
        Self {
            config: Config::from_env_no_version(),
            core_module: CoreModule::builder().build(),
            db_service: MemoryDb::new(),
            api_service: MockApiService::new(),
            lock_service: MockLockService::new(),
        }
    }

    pub fn into_context(self, writer: Arc<RwLock<dyn Write + Send + Sync>>) -> CommandContext {
        CommandContext {
            config: self.config,
            core_module: self.core_module,
            db_service: Box::new(self.db_service),
            api_service: Box::new(self.api_service),
            lock_service: Box::new(self.lock_service),
            writer,
        }
    }
}

pub(crate) async fn test_command(ctx: CommandContextTest, command_args: &[&str]) -> String {
    let buf = Arc::new(RwLock::new(Vec::new()));

    {
        let command_args = {
            let mut tmp_args = vec!["bot"];
            tmp_args.extend(command_args);
            tmp_args
        };

        let args = Args::try_parse_from(command_args);
        match args {
            Ok(args) => CommandExecutor::parse_args_async(args, ctx.into_context(buf.clone()))
                .await
                .unwrap(),
            Err(e) => {
                eprintln!("{}", e);
                panic!("Parse error.")
            }
        }
    }

    let vec = buf.read().await.to_vec();
    std::str::from_utf8(&vec).unwrap().to_string()
}
