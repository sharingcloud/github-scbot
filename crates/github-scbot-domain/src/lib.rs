//! Logic module.

#![warn(clippy::all)]
#![allow(clippy::new_without_default)]

pub mod auth;
pub mod checks;
pub mod commands;
pub mod comments;
pub mod errors;
pub mod external;
pub mod gif;
pub mod pulls;
pub mod reviews;
pub mod status;
pub mod summary;

#[cfg(test)]
mod tests;

pub use errors::{DomainError, Result};
