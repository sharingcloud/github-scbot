//! Distributed lock

#![warn(missing_docs)]
#![warn(clippy::all)]

mod errors;
mod interface;
mod lock_instance;
mod redis;

pub use errors::RedisError;
pub use lock_instance::{LockInstance, LockStatus};
pub use interface::{MockRedisService, RedisService};
pub use redis::RedisServiceImpl;
