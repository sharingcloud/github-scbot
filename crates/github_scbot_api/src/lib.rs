//! API crate.
//!
//! Contains functions to communicate with GitHub API.

#![warn(missing_docs)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::missing_errors_doc,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::must_use_candidate,
    clippy::module_name_repetitions,
    clippy::struct_excessive_bools
)]

pub mod adapter;
pub mod comments;
pub mod errors;
pub mod gif;
pub mod labels;
pub mod reviews;
pub mod utils;

pub use self::errors::{ApiError, Result};
