//! Logic module.

#![warn(clippy::all)]
#![allow(clippy::new_without_default)]

pub mod checks;
pub mod commands;
pub mod comments;
pub mod errors;
pub mod gif;
pub mod pulls;
pub mod reviews;
pub mod status;
pub mod summary;
pub mod use_cases;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod testutils;

pub use errors::{DomainError, Result};
