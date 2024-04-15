//! Server module.

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod admin;
pub mod constants;
mod debug;
pub mod errors;
mod event_type;
mod external;
pub mod ghapi;
mod health;
mod metrics;
pub mod middlewares;
pub mod redis;
pub mod server;
pub mod utils;
mod webhook;

pub use errors::{Result, ServerError};
