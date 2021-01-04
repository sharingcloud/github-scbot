//! SC Bot library

#![deny(missing_docs)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::match_wildcard_for_single_variants,
    clippy::future_not_send,
    clippy::pub_enum_variant_names,
    clippy::default_trait_access
)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

mod api;
mod core;
mod database;
mod errors;
mod server;
mod shell;
mod types;
mod ui;
mod utils;
mod webhook;

pub use shell::initialize_command_line;
