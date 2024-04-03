//! Lock interface

#![warn(missing_docs)]
#![warn(clippy::all)]

mod errors;
mod interface;
mod lock_instance;

pub use errors::LockError;
#[cfg(any(test, feature = "testkit"))]
pub use interface::MockLockService;
pub use interface::{using_lock, LockService, UsingLockResult};
pub use lock_instance::{LockInstance, LockStatus};
