//! `GitHub SharingCloud Bot`.

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod errors;
pub mod shell;

pub use shell::initialize_command_line;
