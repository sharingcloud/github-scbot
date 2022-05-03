//! Types module.

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod checks;
pub mod common;
pub mod errors;
pub mod events;
pub mod issues;
pub mod labels;
pub mod ping;
pub mod pulls;
pub mod repository;
pub mod reviews;
pub mod rule_branch;
pub mod status;

pub use errors::{Result, TypeError};
