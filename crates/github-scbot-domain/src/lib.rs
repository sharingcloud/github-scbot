//! Logic module.

#![warn(clippy::all)]
#![allow(clippy::new_without_default)]

pub mod commands;
pub mod errors;
pub mod use_cases;

pub use errors::{DomainError, Result};

#[cfg(test)]
mod tests;
