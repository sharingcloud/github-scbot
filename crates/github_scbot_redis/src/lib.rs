//! Distributed lock

#![warn(missing_docs)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::missing_errors_doc,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::must_use_candidate,
    clippy::module_name_repetitions,
    clippy::struct_excessive_bools,
    clippy::pub_enum_variant_names
)]

mod dummy;
mod interface;
mod redis;

pub use dummy::DummyRedisAdapter;
pub use interface::{IRedisAdapter, LockInstance, LockStatus, RedisError};
pub use redis::RedisAdapter;
