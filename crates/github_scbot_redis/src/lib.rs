//! Distributed lock

#![warn(missing_docs)]
#![warn(clippy::all)]

mod interface;
mod redis;

pub use interface::{LockInstance, LockStatus, MockRedisService, RedisError, RedisService};
pub use redis::RedisServiceImpl;
