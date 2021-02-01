//! `GitHub SharingCloud Bot`.

#![warn(missing_docs)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::match_wildcard_for_single_variants,
    clippy::future_not_send,
    clippy::pub_enum_variant_names,
    clippy::default_trait_access,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::needless_pass_by_value,
    clippy::type_complexity,
    clippy::must_use_candidate,
    clippy::missing_errors_doc
)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

pub mod api;
pub mod core;
pub mod database;
pub mod errors;
pub mod logic;
pub mod server;
pub mod shell;
pub mod types;
pub mod ui;
pub mod utils;
pub mod webhook;

pub use shell::initialize_command_line;
