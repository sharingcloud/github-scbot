//! Distributed lock

#![warn(missing_docs)]
#![warn(clippy::all)]

mod errors;
mod interface;
mod lock_instance;
mod redis;

pub use errors::LockError;
pub use interface::{LockService, MockLockService};
pub use lock_instance::{LockInstance, LockStatus};
pub use redis::RedisServiceImpl;
