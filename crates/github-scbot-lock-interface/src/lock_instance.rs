use crate::{LockError, LockService};

/// Lock status.
#[derive(Clone, Debug)]
pub enum LockStatus<'a> {
    /// Already locked.
    AlreadyLocked,
    /// Lock successful.
    SuccessfullyLocked(LockInstance<'a>),
}

/// Lock instance.
#[derive(Clone)]
#[must_use]
pub struct LockInstance<'a> {
    pub(crate) lock: Option<&'a dyn LockService>,
    pub(crate) name: String,
}

impl<'a> std::fmt::Debug for LockInstance<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LockInstance")
            .field("name", &self.name)
            .finish()
    }
}

impl<'a> LockInstance<'a> {
    /// Create a new dummy lock.
    pub fn new_dummy<T: Into<String>>(name: T) -> Self {
        Self {
            lock: None,
            name: name.into(),
        }
    }

    /// Create a new lock.
    pub fn new<T: Into<String>>(lock: &'a dyn LockService, name: T) -> Self {
        Self {
            lock: Some(lock),
            name: name.into(),
        }
    }

    /// Release lock instance.
    pub async fn release(self) -> Result<(), LockError> {
        if let Some(lock) = self.lock {
            if lock.has_resource(&self.name).await? {
                lock.del_resource(&self.name).await?;
            }
        }

        Ok(())
    }
}
