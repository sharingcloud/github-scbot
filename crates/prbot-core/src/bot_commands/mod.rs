//! Commands module.

mod command;
pub mod commands;
pub(crate) mod executor;
mod parser;

#[cfg(test)]
pub(crate) use commands::tests::CommandContextTest;
pub use commands::{BotCommand, CommandContext};
pub use executor::CommandExecutorInterface;
#[cfg(any(test, feature = "testkit"))]
pub use executor::MockCommandExecutorInterface;
pub use parser::CommandParser;

pub use self::command::{
    AdminCommand, Command, CommandExecutionResult, CommandHandlingStatus, CommandResult,
    ResultAction, UserCommand,
};
