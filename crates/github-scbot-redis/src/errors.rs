use actix::MailboxError;
use thiserror::Error;

/// Lock error.
#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum RedisError {
    /// Mailbox error.
    #[error("Actix mailbox error,\n  caused by: {}", source)]
    MailboxError { source: MailboxError },

    /// Actix error.
    #[error("Actix-Redis error,\n  caused by: {}", source)]
    ActixError { source: actix_redis::Error },

    /// Command error.
    #[error("Redis command error: {}", result)]
    CommandError { result: String },
}
