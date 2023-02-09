//! Lock interface

#![warn(missing_docs)]
#![warn(clippy::all)]

mod errors;
mod interface;
mod lock_instance;

pub use errors::LockError;
pub use interface::{LockService, MockLockService};
pub use lock_instance::{LockInstance, LockStatus};
