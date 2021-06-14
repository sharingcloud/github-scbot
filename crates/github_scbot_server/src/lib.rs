//! Webhook module.

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

pub mod constants;
pub mod errors;
mod external;
pub mod middlewares;
pub mod server;
pub mod utils;
mod webhook;

pub use errors::{Result, ServerError};
