//! API crate.
//!
//! Contains functions to communicate with GitHub API.

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod adapter;
pub mod auth;
pub mod comments;
pub mod errors;
pub mod gif;
pub mod labels;
pub mod reviews;

pub use self::errors::{ApiError, Result};
