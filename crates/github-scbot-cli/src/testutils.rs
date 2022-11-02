use super::Result;
use clap::Parser;
use github_scbot_core::config::Config;
use github_scbot_database::DbService;
use github_scbot_ghapi::adapter::ApiService;
use github_scbot_redis::RedisService;

use crate::{
    args::{Args, CommandExecutor},
    commands::CommandContext,
};

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
        CommandExecutor::parse_args_async(args, ctx).await?;
    }

    Ok(std::str::from_utf8(buf.as_slice()).unwrap().to_string())
}
