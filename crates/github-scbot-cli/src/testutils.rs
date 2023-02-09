use std::io::Write;

use clap::Parser;
use github_scbot_core::config::Config;
use github_scbot_database::MemoryDb;
use github_scbot_ghapi::adapter::MockApiService;
use github_scbot_lock_interface::MockLockService;

use crate::{
    args::{Args, CommandExecutor},
    commands::CommandContext,
};

pub(crate) struct CommandContextTest {
    pub config: Config,
    pub db_adapter: MemoryDb,
    pub api_adapter: MockApiService,
    pub redis_adapter: MockLockService,
}

impl CommandContextTest {
    pub fn new() -> Self {
        Self {
            config: Config::from_env(),
            db_adapter: MemoryDb::new(),
            api_adapter: MockApiService::new(),
            redis_adapter: MockLockService::new(),
        }
    }

    pub fn into_context<W: Write>(self, writer: W) -> CommandContext<W> {
        CommandContext {
            config: self.config,
            db_adapter: Box::new(self.db_adapter),
            api_adapter: Box::new(self.api_adapter),
            redis_adapter: Box::new(self.redis_adapter),
            writer,
        }
    }
}

pub(crate) async fn test_command(ctx: CommandContextTest, command_args: &[&str]) -> String {
    let mut buf = Vec::new();

    {
        let command_args = {
            let mut tmp_args = vec!["bot"];
            tmp_args.extend(command_args);
            tmp_args
        };

        let args = Args::try_parse_from(command_args);
        match args {
            Ok(args) => CommandExecutor::parse_args_async(args, ctx.into_context(&mut buf))
                .await
                .unwrap(),
            Err(e) => {
                eprintln!("{}", e);
                panic!("Parse error.")
            }
        }
    }

    std::str::from_utf8(buf.as_slice()).unwrap().to_string()
}
