//! Logic module.

pub mod commands;
pub mod database;
pub mod errors;
pub mod reviews;
pub mod status;
pub mod welcome;

#[cfg(test)]
mod tests;

pub use errors::{LogicError, Result};
