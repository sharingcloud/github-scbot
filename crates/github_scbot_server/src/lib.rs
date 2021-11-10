//! Webhook module.

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod constants;
mod debug;
pub mod errors;
mod external;
pub mod middlewares;
pub mod server;
pub mod utils;
mod webhook;

pub use errors::{Result, ServerError};
