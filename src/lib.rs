//! SC Bot library

#![deny(missing_docs)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::module_name_repetitions,
    clippy::match_wildcard_for_single_variants,
    clippy::future_not_send,
    clippy::pub_enum_variant_names
)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

mod api;
mod database;
mod server;
mod utils;
mod webhook;

pub use server::run_bot_server;
pub use utils::configure_startup;
