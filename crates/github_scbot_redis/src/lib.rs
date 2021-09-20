//! Distributed lock

#![warn(missing_docs)]
#![warn(clippy::all)]

mod dummy;
mod interface;
mod redis;

pub use dummy::DummyRedisAdapter;
pub use interface::{IRedisAdapter, LockInstance, LockStatus, RedisError};
pub use redis::RedisAdapter;
