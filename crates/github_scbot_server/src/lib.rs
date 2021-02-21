//! Webhook module.

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod constants;
pub mod errors;
mod external;
pub mod middlewares;
pub mod server;
pub mod utils;
mod webhook;

#[cfg(test)]
mod tests;

pub use errors::{Result, ServerError};
