//! Logic module.

#![warn(missing_docs)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::missing_errors_doc,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::must_use_candidate,
    clippy::module_name_repetitions,
    clippy::struct_excessive_bools,
    clippy::pub_enum_variant_names
)]

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
pub mod validation;
pub mod welcome;

#[cfg(test)]
mod tests;

pub use errors::{LogicError, Result};
