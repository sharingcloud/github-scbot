//! Distributed lock

#![warn(missing_docs)]
#![warn(clippy::all)]

mod redis;

pub use redis::RedisServiceImpl;
