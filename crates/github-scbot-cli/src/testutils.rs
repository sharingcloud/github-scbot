use std::io::Write;

use super::Result;
use clap::Parser;
use github_scbot_core::config::Config;
use github_scbot_database::{DbService, MockDbService};
use github_scbot_ghapi::adapter::{ApiService, MockApiService};
use github_scbot_redis::{MockRedisService, RedisService};

use crate::{
    args::{Args, CommandExecutor},
    commands::CommandContext,
};

pub(crate) struct CommandContextTest {
    pub config: Config,
    pub db_adapter: MockDbService,
    pub api_adapter: MockApiService,
    pub redis_adapter: MockRedisService,
}

impl CommandContextTest {
    pub fn new() -> Self {
        Self {
            config: Config::from_env(),
            db_adapter: MockDbService::new(),
            api_adapter: MockApiService::new(),
            redis_adapter: MockRedisService::new(),
        }
    }

    pub fn to_context<W: Write>(self, writer: W) -> CommandContext<W> {
        CommandContext {
            config: self.config,
            db_adapter: Box::new(self.db_adapter),
            api_adapter: Box::new(self.api_adapter),
            redis_adapter: Box::new(self.redis_adapter),
            writer,
        }
    }
}

pub(crate) async fn test_command(ctx: CommandContextTest, command_args: &[&str]) -> Result<String> {
    let mut buf = Vec::new();

    {
        let command_args = {
            let mut tmp_args = vec!["bot"];
            tmp_args.extend(command_args);
            tmp_args
        };

        let args = Args::try_parse_from(command_args).unwrap();
        CommandExecutor::parse_args_async(args, ctx.to_context(&mut buf)).await?;
    }

    Ok(std::str::from_utf8(buf.as_slice()).unwrap().to_string())
}

pub(crate) fn buffer_to_string(buf: Vec<u8>) -> String {
    std::str::from_utf8(buf.as_slice()).unwrap().to_string()
}
