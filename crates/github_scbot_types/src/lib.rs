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
pub mod pull_requests;
pub mod push;
pub mod status;

#[cfg(test)]
mod tests;
