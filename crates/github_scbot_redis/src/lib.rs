//! Distributed lock

#![warn(missing_docs)]
#![warn(clippy::all)]

mod interface;
mod redis;

pub use interface::{RedisService, MockRedisService, LockInstance, LockStatus, RedisError};
pub use redis::RedisServiceImpl;
