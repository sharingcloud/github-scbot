//! API crate.
//!
//! Contains functions to communicate with GitHub API.

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod auth;
pub mod checks;
pub mod comments;
pub mod errors;
pub mod gif;
pub mod labels;
pub mod pulls;
pub mod reviews;
pub mod status;
pub mod utils;

pub use self::errors::{APIError, Result};
