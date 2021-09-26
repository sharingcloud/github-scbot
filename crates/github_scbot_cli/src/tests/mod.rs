use github_scbot_api::adapter::DummyAPIAdapter;
use github_scbot_conf::Config;
use github_scbot_database::models::DummyDatabaseAdapter;
use github_scbot_redis::DummyRedisAdapter;

use crate::commands::CommandContext;

// pub(crate) fn create_test_context<'a>() -> CommandContext<'a> {
//     let config = Config::from_env();
//     let api_adapter = DummyAPIAdapter::new();
//     let db_adapter = DummyDatabaseAdapter::new();
//     let redis_adapter = DummyRedisAdapter::new();

//     CommandContext {
//         config,
//         pool:
//     }
// }
