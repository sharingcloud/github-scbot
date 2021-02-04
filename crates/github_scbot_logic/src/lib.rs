//! Logic module.

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod checks;
pub mod commands;
pub mod comments;
pub mod database;
pub mod errors;
pub mod pull_requests;
pub mod reviews;
pub mod status;
pub mod welcome;

#[cfg(test)]
mod tests;

pub use errors::{LogicError, Result};
