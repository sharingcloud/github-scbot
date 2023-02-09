//! API crate.
//!
//! Contains functions to communicate with GitHub API.

#![warn(missing_docs)]
#![warn(clippy::all)]

mod auth;
mod errors;
mod github;

pub use github::GithubApiService;
